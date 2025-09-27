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

            match object_table.get(&object_id)? {
                None => {
                    let object_name = &list_response.name[index];
                    println!("[{}] <!> [{}]", prototools::object_id_hex(&object_id), object_name);
                },
                Some(object) => {
                    let object_is_directory = &list_response.is_directory[index];
                    let path = PathBuf::from(object.value());

                    println!("[{}] <-> [{}]", prototools::object_id_hex(&object_id), object.value());

                    if *object_is_directory {
                        recursive_tui(&path, &path, &list_response, &"");
                        println!();
                        continue;
                    }
                }
            }
        }
        println!();

        Ok(())
    }
}

fn recursive_tui(top_tree: &PathBuf, directory: &PathBuf, list_response: &ListResponse, prefix: &str) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => {
            let mut vec: Vec<_> = entries.filter_map(Result::ok).collect();
            vec.sort_by_key(|e| !e.path().is_dir());
            vec
        }
        Err(_) => return
    };

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        process_entry(top_tree, entry, list_response, prefix, is_last);
    }
}

fn process_entry(top_tree: &PathBuf, entry: &fs::DirEntry, list_response: &ListResponse, prefix: &str, is_last: bool) {
    let path = entry.path();

    let relative_name = match path.strip_prefix(top_tree) {
        Err(_) => return,
        Ok(name) => match name.to_str() {
            Some(name) => name,
            None => return,
        }
    };

    let branch = if is_last { "└── " } else { "├── " };

    for (i, remote_name) in list_response.name.iter().enumerate() {
        if relative_name == remote_name {
            let object_id = match prototools::parse_object_id(&list_response.object_id[i]) {
                Ok(id) => id,
                Err(_) => continue,
            };

            println!(
                "{}{}{}[{}] <-> [{}]",
                prefix,
                branch,
                if path.is_dir() { "📁" } else { "📄" },
                prototools::object_id_hex(&object_id),
                relative_name
            );

            if path.is_dir() {
                recursive_tui(top_tree, &path, list_response, &format!("{}{}", prefix, if is_last { "    " } else { "│   " }));
            }
            return;
        }
    }

    println!(
        "{}{}{}[????????] <!> [{}]",
        prefix,
        branch,
        if path.is_dir() { "📁" } else { "📄" },
        relative_name
    );
}