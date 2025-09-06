use std::{env, fs};
use std::fs::File;
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use clap::Parser;
use redb::Database;
use xxhash_rust::xxh3::xxh3_64;
use crate::cli::{ApplicationArguments, Commands, ServerCommands};
use crate::config::Config;
use crate::network::{client, packet};
use crate::proto::comms::object::Add;
use crate::proto::comms::object::remove::Identifier::Path;
use crate::proto::constant::PacketKind;

mod network;
mod cli;
mod config;

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
    println!("{:?}", args.command);
    println!("hello world");

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

    let mut stream = match client::connect(Ipv4Addr::from_str("127.0.0.1").unwrap(), 6342, application_running.clone(), Arc::clone(&config), Arc::clone(&database)) {
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

                        if path.is_dir() {
                            println!("{} is a directory", path.display());

                            let add_packet = Add {
                                hostname: String::from("todo!"),
                                is_directory: path.is_dir(),
                                parent_tree: None,
                                child_of_tree: false,
                                path: path.to_str().unwrap().to_string(),
                                hash: xxh3_64(path.to_str().unwrap().as_bytes()).to_le_bytes().to_vec(),
                            };

                            packet::send_packet(&mut stream, &mut [PacketKind::ObjectAdd as u8], add_packet).unwrap();
                        }
                        // packet::send_packet();
                    }
                    _ => todo!()
                }
            },
            _ => todo!()
        }
    }

    while application_running.load(Ordering::SeqCst) {
    }
}