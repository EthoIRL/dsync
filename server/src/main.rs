use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use proto::config;
use proto::config::ServerConfig;
use proto::state::State;
use crate::network::server;

mod network;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    let current_directory = env::current_dir().expect("failed to get current directory.");

    let config: ServerConfig = match config::load_config(current_directory.join("config.toml")) {
        Ok(config) => config,
        Err(err) => {
            error!("failed to load config");
            error!(err);
            return;
        }
    };
    
    let secret = config.shared_secret.clone();
    let server_state = Arc::new(State::new(config, &secret));

    setup_exit_handler(server_state.clone());

    let ip = server_state.config.ip.unwrap_or_else(|| IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let port = server_state.config.port.clone();

    if let Err(err) = server::start_listening(ip.clone(), port, server_state) {
        error!("failed to start server listen (IP: {}, Port: {})", ip, port);
        error!(err);
        return;
    }
}

pub fn setup_exit_handler(state: Arc<ServerState>) {
    ctrlc::set_handler(move || {
        state.running.store(false, Ordering::SeqCst);
        info!("Shutting down gracefully...");
    }).expect("Error setting Ctrl-C handler");
}