use crate::cli::{ApplicationArguments, Commands};
use crate::commands::{local, remote};
use crate::config::Config;
use crate::network::tools::{protofile, prototools};
use crate::network::{client, packet};
use crate::proto::comms::object::Status;
use crate::proto::comms::List;
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;
use clap::Parser;
use redb::{Database, ReadableDatabase, ReadableTable};
use std::error::Error;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs, thread};

mod network;
mod cli;
mod config;
mod tables;
mod commands;

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
            Commands::Add { path } => {
                if let Err(err) = remote::handle_add(&mut stream, &config, &path) {
                    eprintln!("[*] [DSYNC] Failed to add file or directory ({})", err);
                }
            },
            Commands::Remove { target } => {
                if let Err(err) = remote::handle_remove(&mut stream, &config, &database, &target) {
                    eprintln!("[*] [DSYNC] Failed to remove file or directory ({})", err);
                }
            },
            Commands::Sync { target, local_path } => {
                if let Err(err) = remote::handle_sync(&mut stream, &database, &target, &local_path) {
                    eprintln!("[*] [DSYNC] Failed to sync file or directory ({})", err);
                }
            },
            Commands::Dsync { target } => {
                match local::handle_dsync(&database, &target) {
                    Ok(_) => {
                        println!("[*] [DSYNC] Dsync of [{}] completed successfully", target);
                    },
                    Err(err) => {
                        eprintln!("[*] [DSYNC] Failed to handle dsync command ({})", err);
                    }
                }
            }
            Commands::List => {
                packet::send_packet(&mut stream, &mut [PacketKind::List as u8], List {}).expect("[*] [DSYNC] Error sending packet");
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }

    println!("[*] [DSYNC] Running background polling...");
    println!("[*] [DSYNC] Polling every {}ms", &config.polling_interval);

    while application_running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(config.polling_interval));

        if let Err(err) = query_status_all_objects(&mut stream, &database) {
            println!("[*] [DSYNC] Failed to query all objects ({})", err);
        }
    }
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

            println!("Polling: [{}]", prototools::object_id_hex(&id));
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

            packet::send_packet(stream, &mut [PacketKind::ObjectStatus as u8], status_response)?;
        }
    }

    Ok(())
}