use std::{env, fs};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use crate::config::Config;
use crate::network::client;
use crate::state::ClientState;

mod config;
mod network;
mod state;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    let home_directory = env::home_dir().expect("Home directory not found!");
    if !home_directory.join(".dsync").exists() {
        fs::create_dir(home_directory.join(".dsync")).expect("Failed to create dsync directory in home directory!");
    }
    let config = Config::load_config(home_directory.join(".dsync").join("config.toml")).expect("Failed to load config!");
    let client_state = Arc::new(ClientState::new(config));

    setup_exit_handler(client_state.clone());

    let mut stream = match client::connect(client_state.config.master_ip, client_state.config.master_port, client_state) {
        Ok(stream) => stream,
        Err(err) => {
            error!("Error connecting to master server: {}", err);
            return;
        }
    };
}

pub fn setup_exit_handler(client_state: Arc<ClientState>) {
    ctrlc::set_handler(move || {
        client_state.running.store(false, Ordering::SeqCst);
        info!("Shutting down gracefully...");
    }).expect("Error setting Ctrl-C handler");
}