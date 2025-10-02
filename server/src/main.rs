use tracing_subscriber::FmtSubscriber;

mod config;
mod network;

fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::new())
        .expect("Failed to set default tracing subscriber.");
}
