//! # PubNub Core
//!
//! Core functionality of the PubNub client.
//!
//! The `core` module contains the core functionality of the PubNub client.
//!
//! This module contains the core functionality of the PubNub client. It is
//! intended to be used by the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

#[doc(inline)]
pub use error::PubNubError;
pub mod error;

#[cfg(any(feature = "publish", feature = "access"))]
#[doc(inline)]
pub(crate) use error_response::APIErrorBody;
#[cfg(any(feature = "publish", feature = "access"))]
pub(crate) mod error_response;

#[cfg(feature = "blocking")]
#[doc(inline)]
pub use transport::blocking;
#[doc(inline)]
pub use transport::Transport;
pub mod transport;

#[doc(inline)]
pub use transport_request::{TransportMethod, TransportRequest};
pub mod transport_request;

#[doc(inline)]
pub use transport_response::TransportResponse;
pub mod transport_response;

#[doc(inline)]
pub use serialize::Serialize;

#[doc(inline)]
pub use retry_policy::RequestRetryPolicy;
pub mod retry_policy;

#[cfg(any(feature = "publish", feature = "access"))]
pub mod headers;

pub mod serialize;

#[doc(inline)]
pub use deserializer::Deserializer;
pub mod deserializer;

#[doc(inline)]
pub use serializer::Serializer;
pub mod serializer;

#[doc(inline)]
pub use cryptor::Cryptor;
pub mod cryptor;

#[cfg(feature = "event_engine")]
pub(crate) mod event_engine;

pub(crate) mod metadata;
