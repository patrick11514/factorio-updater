use std::fmt::Display;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::app::{
    api::structs::{Arch, Item, Updates},
    config::Config,
};
pub(crate) mod structs;

#[derive(Clone)]
pub struct Api {
    pub config: Config,
}

impl Config {
    fn to_query(&self) -> String {
        format!("username={}&token={}", self.username, self.token)
    }

    fn to_query_params(&self) -> Vec<(&str, &str)> {
        vec![("username", &self.username), ("token", &self.token)]
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
    ResponseCode(u16),
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
            ApiError::ResponseCode(code) => {
                write!(f, "Unexpected response code: {}", code)
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

    pub async fn get_patches(&self) -> Result<Response<Updates>, ApiError> {
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

    pub async fn get_stable_download_link(
        &self,
        arch: Arch,
        patch: &Item,
    ) -> Result<Option<reqwest::Response>, ApiError> {
        let base_url = format!("{BASE_URL}/get-download");
        let patch = patch.to_string_raw();

        let url = match arch {
            Arch::CoreLinux64 => format!("{base_url}/{patch}/alpha/linux64"),
            Arch::CoreLinuxHeadless64 => format!("{base_url}/{patch}/headless/linux64"),
            Arch::CoreExpansionLinux64 => format!("{base_url}/{patch}/expansion/linux64"),
            Arch::CoreMac => format!("{base_url}/{patch}/alpha/osx"),
            Arch::CoreExpansionMac => format!("{base_url}/{patch}/expansion/osx"),
            Arch::CoreWin64 => format!("{base_url}/{patch}/alpha/win64-manual"),
            Arch::CoreExpansionWin64 => format!("{base_url}/{patch}/expansion/win64-manual"),
            Arch::CoreLinux32 => format!("{base_url}/{patch}/alpha/linux32"),
            Arch::CoreWin32 => format!("{base_url}/{patch}/alpha/win32-manual"),
            _ => panic!("Unsupported architecture for stable download link"),
        };

        let client = reqwest::Client::new();

        log::debug!("Requesting download link from URL: {}", url);

        let res = match client
            .get(url)
            .query(&self.config.to_query_params())
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => return Err(ApiError::Reqwest(err)),
        };

        Ok(match res.status() {
            StatusCode::OK => Some(res),
            StatusCode::NOT_FOUND => None,
            _ => {
                return Err(ApiError::ResponseCode(res.status().as_u16()));
            }
        })
    }
}
