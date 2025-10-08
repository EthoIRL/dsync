use std::sync::atomic::AtomicBool;
use crate::config::Config;

pub struct ClientState {
    pub running: AtomicBool,
    pub config: Config
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        ClientState {
            running: AtomicBool::new(true),
            config
        }
    }
}