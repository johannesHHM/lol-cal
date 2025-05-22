use reqwest::StatusCode;
use std::fmt;

pub mod leagues;
pub mod schedule;

const X_API_KEY_NAME: &str = "x-api-key";
const X_API_KEY_VALUE: &str = "0TvQnueqKa5mxJntVWt0w4LpLfEkrV1Ta8rQBb9Z";

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Request(StatusCode),
    Deserialize(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::Request(e) => write!(f, "Request error: {}", e),
            Error::Deserialize(e) => write!(f, "Deserialize error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Request(_) => None,
            Error::Deserialize(_) => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Http(error)
    }
}
