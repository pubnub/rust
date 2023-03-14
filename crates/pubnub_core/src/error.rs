#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    #[error("Transport error {0}")]
    TransportError(String),
    #[error("Deserialize error: {0}")]
    DeserializeError(String),
    #[error("Serialize error: {0}")]
    SerializeError(String),
}
