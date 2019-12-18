use crate::{Message, Timetoken, Transport};
use async_trait::async_trait;
use http::Uri;
use thiserror::Error;

use mockall::mock;

#[derive(Debug, Error)]
#[error("mock tranport error")]
pub struct MockTransportError;

mock! {
    pub Transport {
        fn sync_publish_request(&self, url: Uri) -> Result<Timetoken, MockTransportError> {}
        fn sync_subscribe_request(&self, url: Uri) -> Result<(Vec<Message>, Timetoken), MockTransportError> {}
    }
    trait Clone {
        fn clone(&self) -> Self;
    }
}

// We implement the mock manually cause `mockall` doesn't support `async_trait` yet.
#[async_trait]
impl Transport for MockTransport {
    type Error = MockTransportError;

    async fn publish_request(&self, url: Uri) -> Result<Timetoken, Self::Error> {
        self.sync_publish_request(url)
    }
    async fn subscribe_request(&self, url: Uri) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        self.sync_subscribe_request(url)
    }
}
