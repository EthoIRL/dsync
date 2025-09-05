use std::env;
use std::net::{Ipv4Addr, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use clap::Parser;
use xxhash_rust::xxh3::xxh3_64;
use crate::cli::{ApplicationArguments, Commands, ServerCommands};
use crate::network::{client, packet};
use crate::proto::comms::object::Add;
use crate::proto::constant::PacketKind;

mod network;
mod cli;

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

    let mut stream = match client::connect(Ipv4Addr::from_str("127.0.0.1").unwrap(), 6342, application_running.clone()) {
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