#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    #[error("Transport error {0}")]
    TransportError(String),
}
