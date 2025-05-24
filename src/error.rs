use reqwest::header;
use serde_json;
use std::{error, fmt, result};
use url;

#[derive(Debug)]
pub struct HackMDError {
    pub message: String,
}

impl fmt::Display for HackMDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for HackMDError {}

#[derive(Debug)]
pub struct HttpResponseError {
    pub message: String,
    pub code: u16,
    pub status_text: String,
}

impl fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl error::Error for HttpResponseError {}

#[derive(Debug)]
pub struct MissingRequiredArgument {
    pub message: String,
}

impl fmt::Display for MissingRequiredArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for MissingRequiredArgument {}

#[derive(Debug)]
pub struct InternalServerError {
    pub message: String,
    pub code: u16,
    pub status_text: String,
}

impl fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl error::Error for InternalServerError {}

#[derive(Debug)]
pub struct TooManyRequestsError {
    pub message: String,
    pub code: u16,
    pub status_text: String,
    pub user_limit: u32,
    pub user_remaining: u32,
    pub reset_after: Option<u64>,
}

impl fmt::Display for TooManyRequestsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}/{} requests remaining",
            self.message, self.code, self.user_remaining, self.user_limit
        )
    }
}

impl error::Error for TooManyRequestsError {}

#[derive(Debug)]
pub enum ApiError {
    HackMD(HackMDError),
    HttpResponse(HttpResponseError),
    MissingRequiredArgument(MissingRequiredArgument),
    InternalServer(InternalServerError),
    TooManyRequests(TooManyRequestsError),
    Reqwest(reqwest::Error),
    Url(url::ParseError),
    Header(header::InvalidHeaderValue),
    Serde(serde_json::Error),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::HackMD(err) => write!(f, "HackMD error: {}", err),
            ApiError::HttpResponse(err) => write!(f, "HTTP response error: {}", err),
            ApiError::MissingRequiredArgument(err) => {
                write!(f, "Missing required argument: {}", err)
            }
            ApiError::InternalServer(err) => write!(f, "Internal server error: {}", err),
            ApiError::TooManyRequests(err) => write!(f, "Too many requests: {}", err),
            ApiError::Reqwest(err) => write!(f, "Request error: {}", err),
            ApiError::Url(err) => write!(f, "URL parse error: {}", err),
            ApiError::Header(err) => write!(f, "Header error: {}", err),
            ApiError::Serde(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::Reqwest(error)
    }
}

impl From<url::ParseError> for ApiError {
    fn from(error: url::ParseError) -> Self {
        ApiError::Url(error)
    }
}

impl From<header::InvalidHeaderValue> for ApiError {
    fn from(error: header::InvalidHeaderValue) -> Self {
        ApiError::Header(error)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::Serde(error)
    }
}

pub type Result<T> = result::Result<T, ApiError>;
