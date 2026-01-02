#![allow(dead_code)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::app::api::structs::{Platform, Version};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledVersion {
    pub version: Version,
    pub platform: Platform,
    pub current_version: String,
    pub path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub username: String,
    pub token: String,
    pub installed_versions: HashMap<uuid::Uuid, InstalledVersion>,
}

#[derive(Debug)]
pub enum ConfigError {
    NoConfigDir,
    Read(PathBuf),
    Parse(serde_json::Error),
    CreateDirectory,
    Write(PathBuf),
}

static FOLDER_NAME: &str = "factorio-updater";

impl Config {
    pub fn new(username: String, token: String) -> Self {
        Self {
            username,
            token,
            installed_versions: HashMap::new(),
        }
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

    pub async fn save(&self) -> Result<(), ConfigError> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir,
            None => return Err(ConfigError::NoConfigDir),
        };

        let config_dir = config_dir.join(Path::new(FOLDER_NAME));

        if let Err(_) = fs::create_dir_all(&config_dir).await {
            return Err(ConfigError::CreateDirectory);
        }

        let config_path = config_dir.join("config.json");

        let data = match serde_json::to_string_pretty(self) {
            Ok(data) => data,
            Err(err) => return Err(ConfigError::Parse(err)),
        };

        if let Err(_) = fs::write(&config_path, data).await {
            return Err(ConfigError::Write(config_path));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use insta::assert_json_snapshot;
    use uuid::Uuid;

    #[test]
    fn test_config_new() {
        let username = "testuser".to_string();
        let token = "testtoken".to_string();
        let config = Config::new(username.clone(), token.clone());

        assert_eq!(config.username, username);
        assert_eq!(config.token, token);
        assert!(config.installed_versions.is_empty());
    }

    #[test]
    fn test_config_snapshot() {
        let mut config = Config::new("user".to_string(), "token123".to_string());

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let installed_version = InstalledVersion {
            version: Version::Vanilla,
            platform: Platform::Linux64,
            current_version: "1.1.100".to_string(),
            path: PathBuf::from("/opt/factorio"),
            installed_at: Utc.timestamp_opt(1672531200, 0).unwrap(), // 2023-01-01 00:00:00 UTC
        };

        config.installed_versions.insert(uuid, installed_version);

        assert_json_snapshot!(config);
    }
}
