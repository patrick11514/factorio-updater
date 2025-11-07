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
}
