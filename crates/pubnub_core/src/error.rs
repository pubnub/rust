#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    #[error("Transport error {0}")]
    TransportError(String),
    #[error("Deserialize error: {0}")]
    DeserializationError(String),
    #[error("Serialize error: {0}")]
    SerializationError(String),
}
