//! # Transport Providers Module
//!
//! This module contains the Transport Providers that can be used by [`PubNubClient`].
//! It is intended to be used by the [`pubnub`] crate.
//!
//! [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
//! [`pubnub`]: ../index.html

pub use self::reqwest::TransportReqwest;
pub mod reqwest;
