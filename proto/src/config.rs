use std::error::Error;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub master_ip: IpAddr,
    pub master_port: u16,

    // Any string; can be treated like a password.
    pub shared_secret: Option<String>,

    // This can be overwritten; however uses the server's default.
    pub polling_interval: Option<u32>,
    pub debug: bool,
}

impl Default for ClientConfig {
    fn default() -> ClientConfig {
        ClientConfig {
            master_ip: IpAddr::from_str("127.0.0.1").unwrap(),
            master_port: 6342,
            shared_secret: None,
            polling_interval: None,
            debug: false
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    // This is in milliseconds
    pub client_sync_rate: u32,

    // Any string; can be treated like a password.
    // Forces a AES256 symmetric communication
    pub shared_secret: Option<String>,

    pub ip: Option<IpAddr>,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            client_sync_rate: 2500,
            shared_secret: None,
            ip: None,
            port: 6342
        }
    }
}

pub fn load_config<T: Serialize + for<'a> Deserialize<'a> + Default>(path: PathBuf) -> Result<T, Box<dyn Error>> {
    if !path.exists() {
        let config = T::default();
        save_config(&config, path)?;
        return Ok(config);
    }

    let content = fs::read_to_string(path)?;
    let config: T = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config<T: Serialize + for<'a> Deserialize<'a> + Default>(config: &T, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(path, toml_str)?;
    Ok(())
}