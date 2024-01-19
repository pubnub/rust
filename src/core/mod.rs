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

#[cfg(any(
    feature = "publish",
    feature = "access",
    feature = "subscribe",
    feature = "presence"
))]
pub(crate) mod service_response;

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

// TODO: Retry policy can be implemented for `no_std` subscribe
//      when `no_std` event engine is implemented.
#[cfg(feature = "std")]
#[doc(inline)]
pub use retry_policy::RequestRetryConfiguration;
#[cfg(feature = "std")]
pub mod retry_policy;

#[doc(inline)]
pub use deserializer::Deserializer;
pub mod deserializer;
#[doc(inline)]
pub use deserialize::Deserialize;
pub mod deserialize;

#[doc(inline)]
pub use serializer::Serializer;
pub mod serializer;
#[doc(inline)]
pub use serialize::Serialize;
pub mod serialize;

#[doc(inline)]
pub use crypto_provider::CryptoProvider;
pub mod crypto_provider;

#[doc(inline)]
pub use cryptor::{Cryptor, EncryptedData};
pub mod cryptor;

#[cfg(all(feature = "std", feature = "subscribe"))]
pub(crate) mod event_engine;

#[cfg(all(feature = "std", feature = "subscribe"))]
#[doc(inline)]
pub use runtime::Runtime;
#[cfg(all(feature = "std", feature = "subscribe"))]
pub mod runtime;

#[doc(inline)]
pub use data_stream::DataStream;
pub mod data_stream;

pub(crate) mod utils;

#[doc(inline)]
pub use types::ScalarValue;

#[doc(inline)]
pub(crate) use entity::PubNubEntity;
pub(crate) mod entity;

#[doc(inline)]
pub use channel::Channel;
pub mod channel;

#[doc(inline)]
pub use channel_group::ChannelGroup;
pub mod channel_group;

#[doc(inline)]
pub use channel_metadata::ChannelMetadata;
pub mod channel_metadata;

#[doc(inline)]
pub use uuid_metadata::UuidMetadata;
pub mod uuid_metadata;

pub mod types;
