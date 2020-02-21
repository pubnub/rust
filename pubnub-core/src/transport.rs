use crate::data::{request, response};
use async_trait::async_trait;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
#[async_trait]
pub trait Transport: Clone + Send + Sync {
    /// Transport-specific error type this transport can generate.
    type Error: std::error::Error + Send + Sync;

    /// Send a Publish Request and return the timetoken.
    async fn publish_request(
        &self,
        request: request::Publish,
    ) -> Result<response::Publish, Self::Error>;

    /// Send a Subscribe Request and return the messages received.
    async fn subscribe_request(
        &self,
        request: request::Subscribe,
    ) -> Result<response::Subscribe, Self::Error>;

    /// Send a Set State Request and return the response.
    async fn set_state_request(
        &self,
        request: request::SetState,
    ) -> Result<response::SetState, Self::Error>;

    /// Send a Get State Request and return the state.
    async fn get_state_request(
        &self,
        request: request::GetState,
    ) -> Result<response::GetState, Self::Error>;
}
