//! TODO: Add docs
use super::{transport_response::TransportResponse, PubNubError, TransportRequest};

/// TODO: Add docs
#[async_trait::async_trait]
pub trait Transport {
    /// TODO: Add docs
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError>;
}
