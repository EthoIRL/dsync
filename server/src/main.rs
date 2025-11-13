use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use crate::config::Config;
use crate::network::server;
use crate::state::ServerState;

mod config;
mod network;
mod state;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    let current_directory = env::current_dir().expect("failed to get current directory.");

    let config = match Config::load_config(current_directory.join("config.toml")) {
        Ok(config) => config,
        Err(err) => {
            error!("failed to load config");
            error!(err);
            return;
        }
    };

    let server_state = Arc::new(ServerState::new(config));

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