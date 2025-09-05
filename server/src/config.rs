use std::error::Error;
use std::fs;
use std::path::PathBuf;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};

const OBJECTS_TABLE: TableDefinition<[u8; 4], Vec<u8>> = TableDefinition::new("objects");

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub sync_rate: u32,
    pub data_location: PathBuf,
}

impl Config {
    pub fn new(current_path: &PathBuf) -> Config {
        Config {
            sync_rate: 30,
            data_location: current_path.clone(),
        }
    }
}

fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    if !path.exists() {
        let config = Config::new(&path);
        save_config(&config, path)?;
        return Ok(config);
    }

    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn save_config(config: &Config, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(path, toml_str)?;
    Ok(())
}