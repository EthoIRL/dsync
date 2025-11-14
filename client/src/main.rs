use std::{env, fs, thread};
use std::sync::Arc;
use std::time::Duration;
use tracing::error;
use tracing_subscriber::FmtSubscriber;
use proto::{config, Add, Packet};
use proto::config::ClientConfig;
use proto::state::State;
use crate::network::client;

mod network;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    let home_directory = env::home_dir().expect("Home directory not found!");
    if !home_directory.join(".dsync").exists() {
        fs::create_dir(home_directory.join(".dsync")).expect("Failed to create dsync directory in home directory!");
    }
    let config: ClientConfig = config::load_config(home_directory.join(".dsync").join("config.toml")).expect("Failed to load config!");
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

    let rr = Add {
        data: 0,
        test: [0, 1]
    };
    
    Packet::send(&mut stream, rr, &client_state.aes_cipher);
    
    thread::sleep(Duration::from_secs(10));
}