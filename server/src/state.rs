use std::sync::atomic::AtomicBool;
use crate::config::Config;

pub struct ServerState {
    pub running: AtomicBool,
    pub config: Config
}