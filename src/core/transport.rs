use super::{transport_response::TransportResponse, PubNubError, TransportRequest};

#[async_trait::async_trait]
pub trait Transport {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError>;
}
