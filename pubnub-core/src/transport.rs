use crate::message::{Message, Timetoken};
use async_trait::async_trait;

use http::Uri;

/// Transport abstracts away the underlying mechanism through which the PubNub
/// client communicates with the PubNub network.
#[async_trait]
pub trait Transport: Clone + Send + Sync {
    /// Transport-specific error type this transport can generate.
    type Error: std::error::Error + Send + Sync;

    /// Send a publish request and return the JSON response.
    async fn publish_request(&self, url: Uri) -> Result<Timetoken, Self::Error>;

    /// Send a subscribe request and return the JSON messages received.
    async fn subscribe_request(&self, url: Uri) -> Result<(Vec<Message>, Timetoken), Self::Error>;
}
