use crate::transport_request::TransportRequest;
use crate::transport_response::TransportResponse;

#[async_trait::async_trait]
trait Transport {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, ()>;
}