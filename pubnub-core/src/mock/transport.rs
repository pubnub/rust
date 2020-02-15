//! [`Transport`] mocks.

use crate::data::{message::Message, request, timetoken::Timetoken};
use crate::Transport;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use mockall::mock;

/// A dummy error used by the [`MockTransport`].
#[derive(Debug, Error)]
#[error("mock tranport error")]
pub struct MockTransportError;

mock! {
    pub Transport {
        #[allow(clippy::type_complexity)]
        fn mock_workaround_publish_request_v1(
            &self,
            req: request::PublishV1,
        ) -> Pin<Box<dyn Future<Output = Result<Timetoken, MockTransportError>> + Send + 'static>> {}

        #[allow(clippy::type_complexity)]
        fn mock_workaround_subscribe_request_v2(
            &self,
            req: request::SubscribeV2,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<Message>, Timetoken), MockTransportError>> + Send + 'static>>
        {}
    }
    trait Clone {
        fn clone(&self) -> Self {}
    }
}

// We implement the mock manually cause `mockall` doesn't support `async_trait` yet.
#[async_trait]
impl Transport for MockTransport {
    type Error = MockTransportError;

    async fn publish_request_v1(&self, req: request::PublishV1) -> Result<Timetoken, Self::Error> {
        self.mock_workaround_publish_request_v1(req).await
    }

    async fn subscribe_request_v2(
        &self,
        req: request::SubscribeV2,
    ) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        self.mock_workaround_subscribe_request_v2(req).await
    }
}
