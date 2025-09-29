use std::error::Error;
use std::fs;
use std::net::{IpAddr};
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub master_ip: IpAddr,
    pub master_port: u16,

    // Any string; can be treated like a password.
    pub shared_secret: Option<String>,

    // This can be overwritten; however uses the server's default.
    pub polling_interval: Option<u32>,
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            master_ip: IpAddr::from_str("127.0.0.1").unwrap(),
            master_port: 6342,
            shared_secret: None,
            polling_interval: None,
            debug: false
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