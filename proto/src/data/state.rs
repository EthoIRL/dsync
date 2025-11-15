use aes::cipher::KeyInit;
use aes::Aes256;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;

pub struct State<T: Serialize + for<'a> Deserialize<'a> + Default>  {
    pub running: AtomicBool,
    pub config: T,

    pub aes_cipher: Option<Aes256>
}

impl<T: Serialize + for<'a> Deserialize<'a> + Default> State<T> {
    pub fn new(config: T, shared_secret: &Option<String>) -> Self {
        let aes_cipher = match &shared_secret {
            None => None,
            Some(secret) => {
                let mut sha_hasher = Sha256::new();
                sha_hasher.update(secret.as_bytes());

                let secret_hash = sha_hasher.finalize();

                Some(Aes256::new(&secret_hash))
            }
        };

        State {
            running: AtomicBool::new(true),
            config,
            aes_cipher
        }
    }
}