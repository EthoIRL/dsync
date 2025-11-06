use std::sync::atomic::AtomicBool;
use aes::cipher::KeyInit;
use aes::Aes256;
use sha2::{Digest, Sha256};
use crate::config::Config;

pub struct ServerState {
    pub running: AtomicBool,
    pub config: Config,

    pub aes_cipher: Option<Aes256>,
}

impl ServerState {
    pub fn new(config: Config) -> ServerState {
        let aes_cipher = match &config.shared_secret {
            None => None,
            Some(secret) => {
                let mut sha_hasher = Sha256::new();
                sha_hasher.update(secret.as_bytes());

                let secret_hash = sha_hasher.finalize();

                Some(Aes256::new(&secret_hash))
            }
        };

        ServerState {
            running: AtomicBool::new(true),
            config,
            aes_cipher
        }
    }
}