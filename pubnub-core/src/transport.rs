use crate::data::{message::Message, request, timetoken::Timetoken};
use async_trait::async_trait;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
#[async_trait]
pub trait Transport: Clone + Send + Sync {
    /// Transport-specific error type this transport can generate.
    type Error: std::error::Error + Send + Sync;

    /// Send a Publish Request V1 and return the timetoken.
    async fn publish_request_v1(
        &self,
        request: request::PublishV1,
    ) -> Result<Timetoken, Self::Error>;

    /// Send a Subscribe Request V2 and return the messages received.
    async fn subscribe_request_v2(
        &self,
        request: request::SubscribeV2,
    ) -> Result<(Vec<Message>, Timetoken), Self::Error>;
}
