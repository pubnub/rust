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
pub mod serialize;
