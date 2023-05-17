//! # PubNub Developer Experience
//!
//! This module provides a structures and methods for the [PubNub] realtime messaging service.
//! It is intended to be used by the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html
//! [PubNub]: https://www.pubnub.com/

#[cfg(feature = "access")]
pub mod access;

#[cfg(feature = "publish")]
pub mod publish;

#[cfg(all(feature = "parse_token", feature = "serde"))]
pub use parse_token::parse_token;
#[cfg(all(feature = "parse_token", not(feature = "serde")))]
pub use parse_token::parse_token_with;
#[cfg(feature = "parse_token")]
pub use parse_token::Token;
#[cfg(feature = "parse_token")]
pub mod parse_token;

pub use pubnub_client::{Keyset, PubNubClient, PubNubClientBuilder};
pub mod pubnub_client;
