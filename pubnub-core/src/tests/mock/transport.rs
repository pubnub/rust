use crate::{Message, Timetoken, Transport};
use async_trait::async_trait;
use http::Uri;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use mockall::mock;

#[derive(Debug, Error)]
#[error("mock tranport error")]
pub struct MockTransportError;

mock! {
    pub Transport {
        #[allow(clippy::type_complexity)]
        fn mock_workaround_publish_request(&self, url: Uri) -> Pin<Box<dyn Future<Output = Result<Timetoken, MockTransportError>> + Send + 'static>> {}
        #[allow(clippy::type_complexity)]
        fn mock_workaround_subscribe_request(&self, url: Uri) -> Pin<Box<dyn Future<Output = Result<(Vec<Message>, Timetoken), MockTransportError>> + Send + 'static>> {}
    }
    trait Clone {
        fn clone(&self) -> Self {}
    }
}

// We implement the mock manually cause `mockall` doesn't support `async_trait` yet.
#[async_trait]
impl Transport for MockTransport {
    type Error = MockTransportError;

    async fn publish_request(&self, url: Uri) -> Result<Timetoken, Self::Error> {
        self.mock_workaround_publish_request(url).await
    }
    async fn subscribe_request(&self, url: Uri) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        self.mock_workaround_subscribe_request(url).await
    }
}
