#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    #[error("Transport error")]
    TransportError,
}
