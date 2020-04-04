//! [`Transport`] mocks.

use crate::data::{presence, request, response};
use crate::{transport::Service, Transport};
use futures_core::future::BoxFuture;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use mockall::mock;

/// A dummy error used by the [`MockTransport`].
#[allow(missing_copy_implementations)]
#[derive(Debug, Error)]
#[error("mock tranport error")]
pub struct MockTransportError;

mod gen {
    #![allow(missing_docs)]
    use super::{mock, BoxFuture, MockTransportError};

    mock! {
        pub Transport {
            fn call<TReq: 'static, TRes: 'static>(
                &self,
                request: TReq,
            ) -> BoxFuture<'static, Result<TRes, MockTransportError>> {}
        }
        trait Clone {
            fn clone(&self) -> Self {}
        }
    }
}
pub use gen::*;

// We implement the mocks manually cause `mockall` doesn't play nice with
// `async_trait`.

macro_rules! impl_mock_service {
    ($req:ty, $res:ty) => {
        // This is an expanded `async_trait` implementation.
        // It's manually tailored to simply pass the control to the `mock_call`
        // to avoid issues with generic type arguments inferrence.
        impl Service<$req> for MockTransport {
            type Response = $res;
            type Error = MockTransportError;

            fn call<'life0, 'async_trait>(
                &'life0 self,
                req: $req,
            ) -> Pin<
                Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'async_trait>,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(MockTransport::call(self, req))
            }
        }
    };
}

impl_mock_service![request::Publish, response::Publish];
impl_mock_service![request::Subscribe, response::Subscribe];

impl_mock_service![request::SetState, response::SetState];
impl_mock_service![request::GetState, response::GetState];
impl_mock_service![
    request::HereNow<presence::respond_with::OccupancyOnly>,
    response::HereNow<presence::respond_with::OccupancyOnly>
];
impl_mock_service![
    request::HereNow<presence::respond_with::OccupancyAndUUIDs>,
    response::HereNow<presence::respond_with::OccupancyAndUUIDs>
];
impl_mock_service![
    request::HereNow<presence::respond_with::Full>,
    response::HereNow<presence::respond_with::Full>
];
impl_mock_service![
    request::GlobalHereNow<presence::respond_with::OccupancyOnly>,
    response::GlobalHereNow<presence::respond_with::OccupancyOnly>
];
impl_mock_service![
    request::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>,
    response::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>
];
impl_mock_service![
    request::GlobalHereNow<presence::respond_with::Full>,
    response::GlobalHereNow<presence::respond_with::Full>
];
impl_mock_service![request::WhereNow, response::WhereNow];
impl_mock_service![request::Heartbeat, response::Heartbeat];

impl Transport for MockTransport {
    type Error = MockTransportError;
}
