//! TODO: Add documentation

/// TODO: Add documentation
#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    /// TODO: Add documentation
    #[error("Transport error: {0}")]
    TransportError(String),

    /// TODO: Add documentation
    #[error("Publish error: {0}")]
    PublishError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
