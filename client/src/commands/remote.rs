use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use redb::Database;
use xxhash_rust::xxh3::xxh3_64;
use crate::cli;
use crate::config::Config;
use crate::network::packet;
use crate::network::tools::protofile;
use crate::proto::comms::object::{Add, Status};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub fn handle_add(stream: &mut TcpStream, config: &Arc<Config>, path: &PathBuf) -> Result<(), Box<dyn Error>>{
    if !path.exists() {
        return Err(format!("File or object does not exist. ({})", path.display()).into());
    }

    let path_string = match path.to_str() {
        None => return Err("Failed to parse path to a string!".into()),
        Some(path) => path.to_string()
    };

    let hash = match path.is_dir() {
        true => xxh3_64(path_string.as_bytes()),
        false => protofile::hash_object(&path)?
    };

    let object_size = match path.is_dir() {
        true => None,
        false => Some(File::open(&path_string)?.metadata()?.len())
    };

    let add_packet = Add {
        hostname: config.hostname.clone(),
        is_directory: path.is_dir(),
        parent_tree: None,
        child_of_tree: false,
        path: path_string.clone(),
        hash,
        object_size
    };

    if let Err(err) = packet::send_packet(stream, &mut [PacketKind::ObjectAdd as u8], add_packet) {
        eprintln!("[*] [DSYNC] Failed to send packet ({})", err);
    }

    if path.is_dir() {
        add_recursion_traversal(stream, path.clone(), &path_string, &config);
    }

    Ok(())
}

pub fn handle_sync(stream: &mut TcpStream, database: &Database, target: &String, local_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if local_path.exists() {
        println!("[*] [DSYNC] File or dir [{}] already exists", local_path.display());
    }

    let path_string = match local_path.to_str() {
        None => return Err("Failed to parse path to a string!".into()),
        Some(path) => path.to_string()
    };

    let id = cli::hex_id_to_u8_array(&target)?;

    let write_txn = database.begin_write()?;
    {
        let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
        object_table.insert(&id, path_string)?;
    }
    write_txn.commit()?;

    let status_response = Status {
        object_id: id.to_vec(),
        hash: None,
        modified_last: None
    };

    packet::send_packet(stream, &mut [PacketKind::ObjectStatus as u8], status_response)?;

    Ok(())
}

fn add_recursion_traversal(stream: &mut TcpStream, directory: PathBuf, tree_parent: &String, config: &Arc<Config>) {
    fs::read_dir(&directory).unwrap()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    add_recursion_traversal(stream, entry.path(), tree_parent, config);
                }

                let mut object_len: Option<u64> = None;

                let hash = match entry.path().is_dir() {
                    true => xxh3_64(entry.path().to_str().unwrap().as_bytes()),
                    false => {
                        let mut file = File::open(entry.path()).expect("Failed to open file... during traversal");

                        object_len = Some(file.metadata().unwrap().len());

                        let mut data: Vec<u8> = Vec::new();
                        match file.read_to_end(&mut data) {
                            Err(_) => xxh3_64(entry.path().to_str().unwrap().as_bytes()),
                            Ok(size) => {
                                if size == 0 {
                                    xxh3_64(entry.path().to_str().unwrap().as_bytes())
                                } else {
                                    xxh3_64(data.as_slice())
                                }
                            }
                        }
                    }
                };

                let add_packet = Add {
                    hostname: config.hostname.clone(),
                    is_directory: entry.path().is_dir(),
                    parent_tree: Some(tree_parent.clone()),
                    child_of_tree: true,
                    path: entry.path().to_str().unwrap().to_string(),
                    hash,
                    object_size: object_len
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();
            }
        });
}