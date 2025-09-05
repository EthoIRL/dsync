use std::error::Error;
use std::fs;
use std::path::PathBuf;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};

const OBJECTS_TABLE: TableDefinition<[u8; 4], Vec<u8>> = TableDefinition::new("objects");

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub sync_rate: u32,
    pub port: u16,
}


impl Default for Config {
    fn default() -> Config {
        Config {
            sync_rate: 30,
            port: 6342
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