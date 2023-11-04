use warp::http::uri::Parts;
use warp::hyper::Body;
use warp::{http, Reply};
use warp::http::{HeaderValue, StatusCode};
use warp::reply::Response;

#[derive(Clone, Debug)]
pub enum Error {
    FailedToParse(String, String),
    KeyNotFound(String),
    CouldNotExtract(String)
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::CouldNotExtract(str) => {
                format!("Could not extract as type \"{}\"", str)
            },
            Self::FailedToParse(from, to) => {
                format!("Failed to parse \"{}\" to \"{}\"", from, to)
            },
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
    pub fn knf(key : &str) -> Self {
        Self::KeyNotFound(key.to_string())
    }

    pub fn cne(expected_type : &str) -> Self {
        Self::CouldNotExtract(expected_type.to_string())
    }
}
