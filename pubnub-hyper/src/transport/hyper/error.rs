//! Hyper transport related errors.

use crate::core::json;
use error_iter::ErrorIter;
use thiserror::Error;

/// # Error variants
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error")]
    Configuration(#[from] Configuration),

    /// HTTP error.
    #[error("HTTP error")]
    Http(#[from] http::Error),

    /// Hyper error.
    #[error("Hyper error")]
    Hyper(#[from] hyper::Error),

    /// Invalid UTF-8.
    #[error("Invalid UTF-8")]
    Utf8(#[from] std::str::Utf8Error),

    /// Invalid JSON.
    #[error("Invalid JSON")]
    Json(#[from] json::Error),

    /// Server error.
    #[error("Server responded with error")]
    Server(String),

    /// Unexpected response schema.
    #[error("Unexpected response schema")]
    UnexpectedResponseSchema(json::JsonValue),
}

impl ErrorIter for Error {}

/// Configuration error variants.
#[derive(Debug, Error, Clone, Copy)]
pub enum Configuration {
    /// Returned when the call involving the secret key was issued, but the
    /// secret key was not configured.
    #[error("Secret key is unavailable")]
    SecretKeyUnavailable,
}
