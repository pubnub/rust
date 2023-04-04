//! # Providers module
//!
//! This module contains the Providers that can be used by [`PubNubClient`].
//! It is intended to be used by the [`pubnub`] crate.
//!
//! [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
//! [`pubnub`]: ../index.html

#[cfg(feature = "serde")]
pub mod serialization_serde;

#[cfg(feature = "serde")]
pub mod deserialization_serde;

#[cfg(feature = "aescbc")]
pub mod crypto_aescbc;
