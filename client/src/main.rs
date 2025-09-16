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
use std::{env, fs, thread};
use std::error::Error;
use std::time::Duration;
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

    init_database(&database).expect("[*] [DSYNC] Failed to initialize database tables!");

    let mut stream = match client::connect(config.master_ip, 6342, application_running.clone(), Arc::clone(&config), Arc::clone(&database)) {
        Ok(stream) => stream,
        Err(err) => {
            panic!("[*] [DSYNC] Error connecting to master server: {}", err);
        }
    };

    if let Err(err) = query_status_all_objects(&mut stream, &database) {
        println!("[*] [DSYNC] Failed to query all objects ({})", err);
    }

    if let Some(command) = args.command {
        match command {
            Commands::Remote { command } => {
                match command {
                    ServerCommands::Add { path } => {
                        if !path.exists() {
                            println!("[*] [DSYNC] File or object does not exist. ({})", path.display());
                            return;
                        }

                        let path_string = match path.to_str() {
                            None => {
                                println!("[*] [DSYNC] Failed to parse path to a string!");
                                return;
                            },
                            Some(path) => path.to_string()
                        };

                        let hash = match path.is_dir() {
                            true => xxh3_64(path_string.as_bytes()),
                            false => protofile::hash_object(&path).unwrap()
                        };

                        let object_size = match path.is_dir() {
                            true => None,
                            false => Some({
                                match File::open(&path_string) {
                                    Err(err) => {
                                        eprintln!("[*] [DSYNC] Failed to open file? ({})", err)
                                        return;
                                    },
                                    Ok(file) => {
                                        match file.metadata() {
                                            Err(err) => {
                                                eprintln!("[*] [DSYNC] Failed to get file metadata? ({})", err)
                                                return;
                                            },
                                            Ok(metadata) => metadata.len()
                                        }
                                    }
                                }
                            })
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

                        packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();

                        if path.is_dir() {
                            add_recursion_traversal(&mut stream, path.clone(), &path_string, &config);
                        }
                    },
                    ServerCommands::Sync { target, local_path } => {
                        let local_path_buf = PathBuf::from(&local_path);

                        if local_path_buf.exists() {
                            println!("File or dir {} already exists", local_path_buf.display());
                        }

                        let id = hex_str_to_u8_array_fast(&target);

                        let write_txn = database.begin_write().unwrap();
                        {
                            let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE).unwrap();
                            object_table.insert(&id, local_path).unwrap();
                        }
                        write_txn.commit().unwrap();

                        let status_response = Status {
                            object_id: id.to_vec(),
                            hash: None,
                            modified_last: None
                        };

                        packet::send_packet(&mut stream, &mut [PacketKind::ObjectStatus as u8], status_response).unwrap();
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
        thread::sleep(Duration::from_millis(100));
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

pub fn init_database(database: &Database) -> Result<(), Box<dyn Error>> {
    let write_txn = database.begin_write()?;
    {
        write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
    }
    write_txn.commit()?;

    Ok(())
}

pub fn query_status_all_objects(stream: &mut TcpStream, database: &Database) -> Result<(), Box<dyn Error>> {
    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

    for objects in object_table.iter()? {
        if let Ok(objects) = objects {
            let id = objects.0.value();
            let path = PathBuf::from(objects.1.value());

            let hash = match path.exists() {
                true => Some(protofile::hash_object(&path)?),
                false => None
            };

            let modified_last = match path.exists() {
                true => Some(protofile::object_last_modified(&path)?),
                false => None
            };

            let status_response = Status {
                object_id: id.to_vec(),
                hash,
                modified_last
            };

            packet::send_packet(stream, &mut [PacketKind::ObjectStatus as u8], status_response).unwrap();
        }
    }


    Ok(())
}