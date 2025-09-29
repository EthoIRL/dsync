use std::error::Error;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // This is in milliseconds
    pub client_sync_rate: u32,
    // Any string; can be treated like a password.
    pub shared_secret: Option<String>,

    pub ip: Option<IpAddr>,
    pub port: u16,

    // Establishes a RSA-2048 link
    pub use_encryption: bool
}

impl Default for Config {
    fn default() -> Config {
        Config {
            client_sync_rate: 2500,
            shared_secret: None,
            ip: None,
            port: 6342,
            use_encryption: true
        }
    }
}

impl Config {
    pub fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
        if !path.exists() {
            let config = Config::default();
            Self::save_config(&config, path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_config(config: &Config, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let toml_str = toml::to_string_pretty(config)?;
        fs::write(path, toml_str)?;
        Ok(())
    }
}