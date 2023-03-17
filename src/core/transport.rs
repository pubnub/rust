//! # Transport module
//!
//! This module contains the [`Transport`] trait and the [`TransportRequest`] and [`TransportResponse`] types.
//!
//! You can implement this trait for your own types, or use one of the provided
//! features to use a transport library.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs

use super::{transport_response::TransportResponse, PubNubError, TransportRequest};

/// This trait is used to send requests to the [`PubNub API`].
///
/// You can implement this trait for your own types, or use one of the provided
/// features to use a transport library.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
///
/// # Examples
/// ```
/// use pubnub::{Transport, TransportRequest, TransportResponse, PubNubError};
///
/// struct MyTransport;
///
/// #[async_trait::async_trait]
/// impl Transport for MyTransport {
///    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
///         // Send your request here
///
///         Ok(TransportResponse::default())
///    }
/// }
/// ```
#[async_trait::async_trait]
pub trait Transport {
    /// Send a request to the [`PubNub API`].
    ///
    /// # Errors
    /// Should return an [`PubNubError::TransportError`] if the request cannot be sent.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError>;
}
