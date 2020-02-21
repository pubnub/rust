//! [`Transport`] mocks.

use crate::data::{request, response};
use crate::Transport;
use async_trait::async_trait;
use futures_core::future::BoxFuture;
use thiserror::Error;

use mockall::mock;

/// A dummy error used by the [`MockTransport`].
#[allow(missing_copy_implementations)]
#[derive(Debug, Error)]
#[error("mock tranport error")]
pub struct MockTransportError;

mod gen {
    #![allow(missing_docs)]
    use super::*;

    mock! {
        pub Transport {
            fn mock_workaround_publish_request(
                &self,
                request: request::Publish,
            ) -> BoxFuture<'static, Result<response::Publish, MockTransportError>> {}

            fn mock_workaround_subscribe_request(
                &self,
                request: request::Subscribe,
            ) -> BoxFuture<'static, Result<response::Subscribe, MockTransportError>> {}

            fn mock_workaround_set_state_request(
                &self,
                request: request::SetState,
            ) -> BoxFuture<'static, Result<response::SetState, MockTransportError>> {}

            fn mock_workaround_get_state_request(
                &self,
                request: request::GetState,
            ) -> BoxFuture<'static, Result<response::GetState, MockTransportError>> {}
        }
        trait Clone {
            fn clone(&self) -> Self {}
        }
    }
}
pub use gen::*;

// We implement the mock manually cause `mockall` doesn't support `async_trait` yet.
#[async_trait]
impl Transport for MockTransport {
    type Error = MockTransportError;

    async fn publish_request(
        &self,
        request: request::Publish,
    ) -> Result<response::Publish, Self::Error> {
        self.mock_workaround_publish_request(request).await
    }

    async fn subscribe_request(
        &self,
        request: request::Subscribe,
    ) -> Result<response::Subscribe, Self::Error> {
        self.mock_workaround_subscribe_request(request).await
    }

    async fn set_state_request(
        &self,
        request: request::SetState,
    ) -> Result<response::SetState, Self::Error> {
        self.mock_workaround_set_state_request(request).await
    }

    async fn get_state_request(
        &self,
        request: request::GetState,
    ) -> Result<response::GetState, Self::Error> {
        self.mock_workaround_get_state_request(request).await
    }
}
