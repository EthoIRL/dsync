use std::sync::atomic::AtomicBool;
use crate::config::Config;

pub struct ServerState {
    pub running: AtomicBool,
    pub config: Config
}

impl ServerState {
    pub fn new(config: Config) -> ServerState {
        ServerState {
            running: AtomicBool::new(true),
            config
        }
    }
}