use crate::data::{message::Message, object::Object, request, timetoken::Timetoken};
use async_trait::async_trait;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
#[async_trait]
pub trait Transport: Clone + Send + Sync {
    /// Transport-specific error type this transport can generate.
    type Error: std::error::Error + Send + Sync;

    /// Send a Publish Request and return the timetoken.
    async fn publish_request(&self, request: request::Publish) -> Result<Timetoken, Self::Error>;

    /// Send a Subscribe Request and return the messages received.
    async fn subscribe_request(
        &self,
        request: request::Subscribe,
    ) -> Result<(Vec<Message>, Timetoken), Self::Error>;

    /// Send a Set State Request and return the response.
    async fn set_state_request(&self, request: request::SetState) -> Result<(), Self::Error>;

    /// Send a Get State Request and return the state.
    async fn get_state_request(&self, request: request::GetState) -> Result<Object, Self::Error>;
}
