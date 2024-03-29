//! # Transport module
//!
//! This module contains the [`Transport`] trait and the [`TransportRequest`]
//! and [`TransportResponse`] types.
//!
//! You can implement this trait for your own types, or use one of the provided
//! features to use a transport library.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs

use super::{transport_response::TransportResponse, PubNubError, TransportRequest};
use crate::lib::alloc::boxed::Box;

/// The default base URL for the [`PubNub API`].
/// This is used for the transport layer.
///
/// Use it when you implement the [`Transport`] trait without any proxy.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
pub const PUBNUB_DEFAULT_BASE_URL: &str = "https://ps.pndsn.com";

/// This trait is used to send requests to the [`PubNub API`].
///
/// You can implement this trait for your own types, or use one of the provided
/// features to use a transport library.
///
/// # Examples
/// ```
/// use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
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
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait Transport: Send + Sync {
    /// Send a request to the [`PubNub API`].
    ///
    /// # Errors
    /// Should return an [`PubNubError::Transport`] if the request cannot be
    /// sent.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError>;
}

#[cfg(feature = "blocking")]
pub mod blocking {
    //! # Blocking transport module
    //!
    //! This module contains the [`Transport`] trait and the
    //! [`TransportRequest`] and [`TransportResponse`] types.
    //!
    //! You can implement this trait for your own types, or use one of the
    //! provided features to use a transport library.
    //!
    //! This trait is used for blocking requests.
    //!
    //! [`PubNub API`]: https://www.pubnub.com/docs

    use crate::core::{PubNubError, TransportRequest, TransportResponse};

    /// This trait is used to send requests to the [`PubNub API`].
    ///
    /// You can implement this trait for your own types, or use one of the
    /// provided features to use a transport library.
    ///
    /// This trait is used for blocking requests.
    ///
    /// # Examples
    /// ```
    /// use pubnub::core::{blocking::Transport, TransportRequest, TransportResponse, PubNubError};
    ///
    /// struct MyTransport;
    ///
    /// impl Transport for MyTransport {
    ///    fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
    ///         // Send your request here
    ///
    ///         Ok(TransportResponse::default())
    ///    }
    /// }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub trait Transport {
        /// Send a request to the [`PubNub API`].
        ///
        /// # Errors
        /// Should return an [`PubNubError::Transport`] if the request cannot be
        /// sent.
        ///
        /// [`PubNub API`]: https://www.pubnub.com/docs
        fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError>;
    }
}
