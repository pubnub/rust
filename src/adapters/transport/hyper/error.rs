use error_iter::ErrorIter;
use thiserror::Error;

/// # Error variants
#[derive(Debug, Error)]
pub enum Error {
    /// Hyper error.
    #[error("Hyper error")]
    Hyper(#[from] hyper::Error),

    /// Invalid UTF-8.
    #[error("Invalid UTF-8")]
    Utf8(#[from] std::str::Utf8Error),

    /// Invalid JSON.
    #[error("Invalid JSON")]
    Json(#[from] json::Error),
}

impl ErrorIter for Error {}
