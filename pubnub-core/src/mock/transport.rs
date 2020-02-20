//! [`Transport`] mocks.

use crate::data::{message::Message, object::Object, request, timetoken::Timetoken};
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
            ) -> BoxFuture<'static, Result<Timetoken, MockTransportError>> {}

            fn mock_workaround_subscribe_request(
                &self,
                request: request::Subscribe,
            ) -> BoxFuture<'static, Result<(Vec<Message>, Timetoken), MockTransportError>> {}

            fn mock_workaround_set_state_request(
                &self,
                request: request::SetState,
            ) -> BoxFuture<'static, Result<(), MockTransportError>> {}

            fn mock_workaround_get_state_request(
                &self,
                request: request::GetState,
            ) -> BoxFuture<'static, Result<Object, MockTransportError>> {}
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

    async fn publish_request(&self, request: request::Publish) -> Result<Timetoken, Self::Error> {
        self.mock_workaround_publish_request(request).await
    }

    async fn subscribe_request(
        &self,
        request: request::Subscribe,
    ) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        self.mock_workaround_subscribe_request(request).await
    }

    async fn set_state_request(&self, request: request::SetState) -> Result<(), Self::Error> {
        self.mock_workaround_set_state_request(request).await
    }

    async fn get_state_request(&self, request: request::GetState) -> Result<Object, Self::Error> {
        self.mock_workaround_get_state_request(request).await
    }
}
