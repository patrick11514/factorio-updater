use std::fmt::Display;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::app::{api::structs::Updates, config::Config};
pub(crate) mod structs;

#[derive(Clone)]
pub struct Api {
    pub config: Config,
}

impl Config {
    fn to_query(&self) -> String {
        format!("username={}&token={}", self.username, self.token)
    }
}

const BASE_URL: &str = "https://factorio.com";

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub(crate) message: String,
    pub(crate) status: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Success(T),
    Error(ErrorResponse),
}

pub enum ApiError {
    Reqwest(reqwest::Error),
    Decode(serde_json::Error, String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Reqwest(err) => {
                log::debug!("Reqwest error: {}", err);
                write!(f, "Network request failed")
            }
            ApiError::Decode(err, text) => {
                log::debug!("Decode error: {}", err);
                log::debug!("Response text: {}", text);
                write!(f, "Failed to decode response")
            }
        }
    }
}

impl Api {
    pub fn new(config: Config) -> Self {
        Api { config }
    }

    pub async fn check_credentials(&self) -> Result<bool, ApiError> {
        let url = format!(
            "{}/get-available-versions?{}",
            BASE_URL,
            self.config.to_query()
        );

        let res = match reqwest::get(url).await {
            Ok(res) => res,
            Err(err) => return Err(ApiError::Reqwest(err)),
        };

        Ok(res.status() == StatusCode::OK)
    }

    pub async fn get_versions(&self) -> Result<Response<Updates>, ApiError> {
        let url = format!(
            "{}/get-available-versions?{}",
            BASE_URL,
            self.config.to_query()
        );

        let res = match reqwest::get(url).await {
            Ok(res) => res,
            Err(err) => return Err(ApiError::Reqwest(err)),
        };

        let text = match res.text().await {
            Ok(text) => text,
            Err(err) => return Err(ApiError::Reqwest(err)),
        };

        match serde_json::from_str::<Response<Updates>>(&text) {
            Ok(data) => Ok(data),
            Err(err) => {
                log::debug!("Failed to decode response: {}", err);
                Err(ApiError::Decode(err, text))
            }
        }
    }
}
