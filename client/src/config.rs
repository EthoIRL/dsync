use std::error::Error;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub hostname: String,
    pub master_ip: Ipv4Addr,
    pub master_port: u16,
    pub allow_local_deletion: bool,
    pub polling_interval: u64,
    pub debug: bool
}

impl Default for Config {
    fn default() -> Config {
        Config {
            hostname: String::from("default"),
            master_ip: Ipv4Addr::from_str("127.0.0.1").unwrap(),
            master_port: 6342,
            allow_local_deletion: true,
            polling_interval: 1000,
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