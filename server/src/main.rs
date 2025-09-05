use std::env::current_dir;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use redb::Database;
use crate::config::Config;
use crate::network::server;

mod network;
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
    println!("[*] [DSYNC] - [0.1.0] - [ETHO] [*]");

    let application_running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&application_running);

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
        println!("[*] [DSYNC] Server shutting down gracefully...");
    }).expect("[*] [DSYNC] Error setting Ctrl-C handler");

    let current_directory = match current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            panic!("[*] [DSYNC] Error getting current directory: ({})", err);
        }
    };

    let config = match Config::load_config(current_directory.join("config.toml")) {
        Ok(config) => Arc::new(config),
        Err(err) => {
            panic!("[*] [DSYNC] Error loading config: ({})", err);
        }
    };

    let database = match Database::create(current_directory.join("dsync.db")) {
        Ok(database) => Arc::new(database),
        Err(err) => {
            panic!("[*] [DSYNC] Error creating or opening database: ({})", err);
        }
    };

    if let Err(err) = server::start_listening(Ipv4Addr::UNSPECIFIED, config.port, application_running, config, database) {
        panic!("[*] [DSYNC] Error starting server: ({})", err);
    }
}
