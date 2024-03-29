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

#[cfg(feature = "subscribe")]
pub mod subscribe;

#[cfg(feature = "presence")]
pub mod presence;

#[cfg(all(feature = "parse_token", feature = "serde"))]
pub use parse_token::parse_token;
#[cfg(feature = "parse_token")]
pub use parse_token::{parse_token_with, Token};
#[cfg(feature = "parse_token")]
pub mod parse_token;

pub use pubnub_client::{Keyset, PubNubClientBuilder, PubNubGenericClient};
pub mod pubnub_client;

#[cfg(feature = "reqwest")]
pub use pubnub_client::PubNubClient;
