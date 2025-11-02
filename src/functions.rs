use std::{fs, path::Path};

use anyhow::Context;
use serde_json::json;

use crate::structs::{Args, Config};

pub const API_VERSION: &str = "2";

pub fn get_base_query_params(args: &Args) -> serde_json::Value {
    json!({
        "username": args.username,
        "token": args.token,
        "version": API_VERSION,
    })
}

pub fn load_config(base_folder: &Path) -> anyhow::Result<Option<Config>> {
    if fs::exists(base_folder.join("config.json"))
        .context("Failed to check if config file exists")?
    {
        let config_data = fs::read_to_string(base_folder.join("config.json"))
            .context("Failed to read config file")?;
        let config: Config =
            serde_json::from_str(&config_data).context("Failed to parse config file")?;

        Ok(Some(config))
    } else {
        Ok(None)
    }
}
