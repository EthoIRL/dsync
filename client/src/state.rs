use std::sync::atomic::AtomicBool;
use crate::config::Config;

pub struct ClientState {
    pub running: AtomicBool,
    pub config: Config
}