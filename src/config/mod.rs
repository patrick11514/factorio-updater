use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub username: String,
    pub token: String,
}

#[derive(Debug)]
pub enum ConfigError {
    NoConfigDir,
    Read(PathBuf),
    Parse(serde_json::Error),
    CreateDirectory,
    Write(PathBuf),
}

static FOLDER_NAME: &'static str = "factorio-updater";

impl Config {
    pub fn new(username: String, token: String) -> Self {
        Self { username, token }
    }

    pub async fn load() -> Result<Option<Self>, ConfigError> {
        let config = match dirs::config_dir() {
            Some(dir) => dir,
            None => return Err(ConfigError::NoConfigDir),
        };

        let config = config.join(Path::new(FOLDER_NAME)).join("config.json");

        if !match fs::try_exists(&config).await {
            Ok(exists) => exists,
            Err(_) => return Err(ConfigError::Read(config)),
        } {
            return Ok(None);
        }

        let data = match fs::read_to_string(&config).await {
            Ok(data) => data,
            Err(_) => return Err(ConfigError::Read(config)),
        };

        let config: Config = match serde_json::from_str(&data) {
            Ok(config) => config,
            Err(err) => return Err(ConfigError::Parse(err)),
        };

        Ok(Some(config))
    }

    //TODO MAKE I ASYNC
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir,
            None => return Err(ConfigError::NoConfigDir),
        };

        let config_dir = config_dir.join(Path::new(FOLDER_NAME));

        if let Err(_) = std::fs::create_dir_all(&config_dir) {
            return Err(ConfigError::CreateDirectory);
        }

        let config_path = config_dir.join("config.json");

        let data = match serde_json::to_string_pretty(self) {
            Ok(data) => data,
            Err(err) => return Err(ConfigError::Parse(err)),
        };

        if let Err(_) = std::fs::write(&config_path, data) {
            return Err(ConfigError::Write(config_path));
        }

        Ok(())
    }
}
