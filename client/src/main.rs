use std::{env, fs, thread};
use std::sync::Arc;
use std::time::Duration;
use tracing::error;
use tracing_subscriber::FmtSubscriber;
use proto::{Packet, PROTOCOL_VERSION};
use proto::data::config::ClientConfig;
use proto::data::config;
use proto::data::state::State;
use proto::packets::handshake::Handshake;
use proto::packets::object_add::Add;
use crate::network::client;

mod network;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    let home = match env::home_dir() {
        Some(home) => home,
        None => {
            error!("Couldn't find home directory, see https://doc.rust-lang.org/std/env/fn.home_dir.html");
            return;
        }
    };
    
    if !home.join(".dsync").exists() {
        if fs::create_dir(home.join(".dsync")).is_err() {
            error!("Failed to create .dsync directory in your home folder...");
            return;
        }
    }
    
    let config: ClientConfig = match config::load_config(home.join(".dsync").join("config.toml")) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load config: {:#?}", err);
            return;
        }
    };
    
    let secret = config.shared_secret.clone();
    let client_state = Arc::new(State::new(config, &secret));

    proto::setup_exit_handler(client_state.clone());

    let mut stream = match client::connect(client_state.config.master_ip, client_state.config.master_port, client_state.clone()) {
        Ok(stream) => stream,
        Err(err) => {
            error!("Error connecting to master server: {}", err);
            return;
        }
    };
    
    
    let connection_handshake = Handshake {
        version: PROTOCOL_VERSION
    };

    Packet::send(&mut stream, connection_handshake, &client_state.aes_cipher);

    let rr = Add {
        data: 0,
        test: [0, 1]
    };
    
    Packet::send(&mut stream, rr, &client_state.aes_cipher);
    
    thread::sleep(Duration::from_secs(10));
}