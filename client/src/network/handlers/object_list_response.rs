use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::ListResponse;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase};
use std::error::Error;
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ObjectListResponse;

impl GenericHandler for ObjectListResponse {
    fn handle(_: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let list_response: ListResponse = packet.decode()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        for index in 0..list_response.object_id.len() {
            let object_is_child = &list_response.is_child[index];

            if *object_is_child {
                continue
            }

            let object_id = prototools::parse_object_id(&list_response.object_id[index])?;

            if let Some(object) = object_table.get(&object_id)? {
                let path = PathBuf::from(object.value());

                println!("[{}] => [{}]", prototools::object_id_hex(&object_id), object.value());

                if path.is_dir() {
                    recursive_tui(&path, &path, &list_response, 0);
                    println!("<-------------------------------------->");
                    continue;
                }
            }
        }
        println!();

        Ok(())
    }
}

fn recursive_tui(top_tree: &PathBuf, directory: &PathBuf, list_response: &ListResponse, depth: usize) {
    let entries = match fs::read_dir(directory) {
        Ok(e) => e.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("[*] [DSYNC] Error reading directory {}: {}", directory.display(), e);
            return;
        }
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(entry);
        } else {
            files.push(entry);
        }
    }

    dirs.sort_by_key(|e| e.file_name());
    files.sort_by_key(|e| e.file_name());

    for dir in dirs {
        process_entry(top_tree, &dir, list_response, depth);
    }

    for file in files {
        process_entry(top_tree, &file, list_response, depth);
    }
}

fn process_entry(top_tree: &PathBuf, entry: &fs::DirEntry, list_response: &ListResponse, depth: usize) {
    let path = entry.path();

    let relative_name = match path.strip_prefix(top_tree) {
        Err(_) => return,
        Ok(name) => match name.to_str() {
            Some(name) => name,
            None => return,
        }
    };

    let mut is_synced = false;

    for (i, remote_name) in list_response.name.iter().enumerate() {
        if relative_name == remote_name {
            let object_id = match prototools::parse_object_id(&list_response.object_id[i]) {
                Ok(id) => id,
                Err(_) => continue,
            };

            println!(
                "|  {}[{}] => [{}]",
                "    ".repeat(depth),
                prototools::object_id_hex(&object_id),
                relative_name
            );

            is_synced = true;
            break;
        }
    }

    if is_synced {
        if path.is_dir() {
            recursive_tui(top_tree, &path, list_response, depth + 1);
        }
    } else {
        println!("|  {}[????????] => [{}]", "    ".repeat(depth), relative_name);
    }
}