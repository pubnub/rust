//! [`Transport`] mocks.

use crate::data::{request, response};
use crate::{transport::Service, Transport};
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

// We implement the mocks manually cause `mockall` doesn't support `async_trait` yet.

#[async_trait]
impl Service<request::Publish> for MockTransport {
    type Response = response::Publish;
    type Error = MockTransportError;

    async fn call(&self, req: request::Publish) -> Result<Self::Response, Self::Error> {
        self.mock_workaround_publish_request(req).await
    }
}

#[async_trait]
impl Service<request::Subscribe> for MockTransport {
    type Response = response::Subscribe;
    type Error = MockTransportError;

    async fn call(&self, req: request::Subscribe) -> Result<Self::Response, Self::Error> {
        self.mock_workaround_subscribe_request(req).await
    }
}

#[async_trait]
impl Service<request::SetState> for MockTransport {
    type Response = response::SetState;
    type Error = MockTransportError;

    async fn call(&self, req: request::SetState) -> Result<Self::Response, Self::Error> {
        self.mock_workaround_set_state_request(req).await
    }
}

#[async_trait]
impl Service<request::GetState> for MockTransport {
    type Response = response::GetState;
    type Error = MockTransportError;

    async fn call(&self, req: request::GetState) -> Result<Self::Response, Self::Error> {
        self.mock_workaround_get_state_request(req).await
    }
}

impl Transport for MockTransport {
    type Error = MockTransportError;
}
