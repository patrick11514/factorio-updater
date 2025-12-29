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
    message: String,
    statuc: u32,
}

type Response<T> = Result<T, ErrorResponse>;

pub enum ApiError {
    Reqwest,
    Decode,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Reqwest => write!(f, "Network request failed"),
            ApiError::Decode => write!(f, "Failed to decode response"),
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
            Err(_) => return Err(ApiError::Reqwest),
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
            Err(_) => return Err(ApiError::Reqwest),
        };

        res.json().await.map_err(|_| ApiError::Decode)
    }
}
