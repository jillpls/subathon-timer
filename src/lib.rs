use serde::{Deserialize, Serialize};
use warp::http::{HeaderValue, StatusCode};
use warp::reply::Response;
use warp::{http, Reply};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct EventCounts {
    pub subs: u64,
    pub donations: f64,
    pub bits: u64,
    pub channel_point_rewards: u64,
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub kofi_ratio: f64,
    pub subscription_value: f64,
    pub bit_per_100_value: f64,
    pub per_channel_point_reward: f64,
}

impl Default for Settings {
    fn default() -> Self {
       Self {
           kofi_ratio : 1.0,
           subscription_value : 4.0,
           bit_per_100_value : 1.0,
           per_channel_point_reward : 1.0
       }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    FailedToParse(String, String),
    KeyNotFound(String),
    CouldNotExtract(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::CouldNotExtract(str) => {
                format!("Could not extract as type \"{}\"", str)
            }
            Self::FailedToParse(from, to) => {
                format!("Failed to parse \"{}\" to \"{}\"", from, to)
            }
            Self::KeyNotFound(key) => {
                format!("Key not found: \"{}\"", key)
            }
        }
    }
}

impl Reply for Error {
    fn into_response(self) -> Response {
        let mut response = Response::new(self.to_string().into());
        *response.status_mut() = StatusCode::from_u16(503).unwrap_or(StatusCode::default());
        response.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str("text/plain").unwrap(),
        );
        response
    }
}

impl Error {
    pub fn ftp(from: &str, to: &str) -> Self {
        Self::FailedToParse(from.to_string(), to.to_string())
    }
    pub fn knf(key: &str) -> Self {
        Self::KeyNotFound(key.to_string())
    }

    pub fn cne(expected_type: &str) -> Self {
        Self::CouldNotExtract(expected_type.to_string())
    }
}
