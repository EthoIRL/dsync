use std::thread;
use tracing::{event, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod network;
mod state;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");

    thread::spawn(|| {
        event!(Level::WARN, "Multithread log")
    });
}