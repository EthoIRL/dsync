use crate::config::Config;
use aes::cipher::KeyInit;
use aes::Aes256;
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;

pub struct ClientState {
    pub running: AtomicBool,
    pub config: Config,

    pub aes_cipher: Option<Aes256>,
}

impl ClientState {
    pub fn new(config: Config) -> ClientState {
        let aes_cipher = match &config.shared_secret {
            None => None,
            Some(secret) => {
                let mut sha_hasher = Sha256::new();
                sha_hasher.update(secret.as_bytes());

                let secret_hash = sha_hasher.finalize();

                Some(Aes256::new(&secret_hash))
            }
        };

        ClientState {
            running: AtomicBool::new(true),
            config,
            aes_cipher
        }
    }
}