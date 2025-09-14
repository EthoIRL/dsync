use crate::cli::{ApplicationArguments, ClientCommands, Commands, ServerCommands};
use crate::config::Config;
use crate::network::tools::protofile;
use crate::network::{client, packet};
use crate::proto::comms::object::{Add, Status};
use crate::proto::comms::List;
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;
use clap::Parser;
use redb::{Database, ReadableDatabase, ReadableTable};
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, fs};
use xxhash_rust::xxh3::xxh3_64;

mod network;
mod cli;
mod config;
mod tables;

pub mod proto {
    pub mod comms {
        include!(concat!(env!("OUT_DIR"), "/comms.rs"));
    }
    pub mod constant {
        include!(concat!(env!("OUT_DIR"), "/constant.rs"));
    }
}

fn main() {
    let args = ApplicationArguments::parse();

    let application_running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&application_running);

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
        println!("[*] [DSYNC] Client shutting down gracefully...");
    }).expect("[*] [DSYNC] Error setting Ctrl-C handler");

    let home_directory = env::home_dir().expect("[*] [DSYNC] Home directory not found!");
    if !home_directory.join(".dsync").exists() {
        fs::create_dir(home_directory.join(".dsync")).expect("[*] [DSYNC] Failed to create dsync directory in home directory!");
    }
    let config = Arc::new(Config::load_config(home_directory.join(".dsync").join("config.toml")).expect("[*] [DSYNC] Failed to load config!"));

    let database = match Database::create(home_directory.join(".dsync").join("dsync.db")) {
        Ok(database) => Arc::new(database),
        Err(err) => {
            panic!("[*] [DSYNC] Error creating or opening local database: ({})", err);
        }
    };

    let write_txn = database.begin_write().unwrap();
    {
        write_txn.open_table(OBJECTS_LOCAL_TABLE).unwrap();
    }
    write_txn.commit().unwrap();

    let mut stream = match client::connect(config.master_ip, 6342, application_running.clone(), Arc::clone(&config), Arc::clone(&database)) {
        Ok(stream) => stream,
        Err(err) => {
            panic!("[DSYNC] Error connecting to master server: {}", err);
        }
    };

    // TODO: On startup sync all objects using PacketKind::Status
    // Query all paths in the database
    // Use functions inside client/Object_sync.rs
    // Hash & Last modified

    let read_txn = database.begin_read().unwrap();
    let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE).unwrap();
    for objects in object_table.iter().unwrap() {
        if let Ok(objects) = objects {
            let id = objects.0.value();
            let path = PathBuf::from(objects.1.value());

            let hash = match path.exists() {
                true => Some(protofile::hash_object(&path).unwrap()),
                false => None
            };

            let modified_last = match path.exists() {
                true => Some(protofile::object_last_modified(&path).unwrap()),
                false => None
            };

            let status_response = Status {
                object_id: id.to_vec(),
                hash,
                modified_last
            };

            packet::send_packet(&mut stream, &mut [PacketKind::ObjectStatus as u8], status_response).unwrap();
        }
    }

    if let Some(command) = args.command {
        match command {
            Commands::Remote { command } => {
                match command {
                    ServerCommands::Add { path } => {
                        if !path.exists() {
                            println!("File {} does not exist", path.display());
                            return;
                        }

                        let string_path = path.to_str().unwrap().to_string();

                        if path.is_dir() {
                            let add_packet = Add {
                                hostname: config.hostname.clone(),
                                is_directory: path.is_dir(),
                                parent_tree: None,
                                child_of_tree: false,
                                path: string_path.clone(),
                                hash: xxh3_64(path.to_str().unwrap().as_bytes()),
                                object_size: None
                            };

                            packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();

                            add_recursion_traversal(&mut stream, path.clone(), &string_path, &config);
                        } else {
                            let size = File::open(&string_path).unwrap().metadata().unwrap().len();
                            let string_path = path.to_str().unwrap().to_string();
                            let object_hash = protofile::hash_object(&path).unwrap();

                            let add_packet = Add {
                                hostname: config.hostname.clone(),
                                is_directory: path.is_dir(),
                                parent_tree: None,
                                child_of_tree: false,
                                path: string_path.clone(),
                                hash: object_hash,
                                object_size: Some(size)
                            };

                            packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();
                        }
                    },
                    ServerCommands::Sync { target, local_path } => {
                        println!("TODO: {target} {local_path}");
                        todo!()
                    },
                    ServerCommands::Remove { target } => {
                        todo!()
                    }
                }
            },
            Commands::List => {
                packet::send_packet(&mut stream, &mut [PacketKind::List as u8], List {}).expect("[*] [DSYNC] Error sending packet");
            },
            Commands::Local { command } => {
                match command {
                    ClientCommands::Dsync { target } => {
                        // Todo: Right now we're just going to assume target is an 4 u8 ID for ease of testing.
                        // Todo: We should loop the table and check if any paths link later (To obtain the id)

                        let id = hex_str_to_u8_array_fast(&target);

                        let write_txn = database.begin_write().unwrap();
                        {
                            let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE).unwrap();
                            match object_table.remove(&id) {
                                Ok(path) => {
                                    match path {
                                        Some(path) => {
                                            println!("[*] [DSYNC] [{}] Dsync from master server. [{:?}]", path.value(), id);
                                        },
                                        None => println!("[*] [DSYNC] Couldn't find file locally..")
                                    }
                                },
                                Err(_) => println!("[*] [DSYNC] Error while trying to find file locally..")
                            };
                        }
                        write_txn.commit().unwrap();

                        todo!()
                    },
                    ClientCommands::Resync { target } => {
                        todo!()
                    }
                }
            },
            _ => todo!()
        }
    }

    while application_running.load(Ordering::SeqCst) {
    }
}

fn hex_str_to_u8_array_fast(hex: &str) -> [u8; 4] {
    let bytes = hex.as_bytes();
    [
        u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(&bytes[0..2]) }, 16).unwrap(),
        u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(&bytes[2..4]) }, 16).unwrap(),
        u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(&bytes[4..6]) }, 16).unwrap(),
        u8::from_str_radix(unsafe { std::str::from_utf8_unchecked(&bytes[6..8]) }, 16).unwrap(),
    ]
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