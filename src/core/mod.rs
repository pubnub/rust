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

pub use error::PubNubError;
pub mod error;

pub use transport::Transport;
pub mod transport;

pub use transport_request::{TransportMethod, TransportRequest};
pub mod transport_request;

pub use transport_response::TransportResponse;
pub mod transport_response;

pub use serialize::Serialize;
pub mod serialize;
