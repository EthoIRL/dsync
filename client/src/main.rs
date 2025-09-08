use std::{env, fs, thread};
use std::fs::File;
use std::io::Read;
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use clap::Parser;
use redb::Database;
use xxhash_rust::xxh3::xxh3_64;
use crate::cli::{ApplicationArguments, Commands, ServerCommands};
use crate::config::Config;
use crate::network::{client, packet};
use crate::network::handlers::object_status::hash_object;
use crate::proto::comms::List;
use crate::proto::comms::object::Add;
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;

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
                            };

                            packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();

                            add_recursion_traversal(&mut stream, path.clone(), &string_path, &config);
                        } else {
                            let string_path = path.to_str().unwrap().to_string();
                            let object_hash = hash_object(&path).unwrap();

                            let add_packet = Add {
                                hostname: config.hostname.clone(),
                                is_directory: path.is_dir(),
                                parent_tree: None,
                                child_of_tree: false,
                                path: string_path.clone(),
                                hash: object_hash,
                            };

                            packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();
                        }
                    },
                    ServerCommands::Sync { target, local_path } => {
                        println!("TODO: {target} {local_path}");
                    },
                    _ => todo!()
                }
            },
            Commands::List => {
                packet::send_packet(&mut stream, &mut [PacketKind::List as u8], List {}).expect("[*] [DSYNC] Error sending packet");
            }
            _ => todo!()
        }
    }

    while application_running.load(Ordering::SeqCst) {
    }
}

fn add_recursion_traversal(stream: &mut TcpStream, directory: PathBuf, tree_parent: &String, config: &Arc<Config>) {
    fs::read_dir(&directory).unwrap()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    add_recursion_traversal(stream, entry.path(), tree_parent, config);
                }

                let hash = match entry.path().is_dir() {
                    true => xxh3_64(entry.path().to_str().unwrap().as_bytes()),
                    false => {
                        let mut file = File::open(entry.path()).expect("Failed to open file... during traversal");

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
                    hash
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();
            }
        });
}