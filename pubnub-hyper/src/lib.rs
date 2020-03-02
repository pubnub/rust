//! # PubNub Hyper
//!
//! A PubNub client using [`hyper`](hyper) and [`tokio`](tokio) to provide an
//! ultra-fast, incredibly reliable message transport over the PubNub edge
//! network.
//!
//! Uses `pubnub-core`(pubnub_core) under the hood.
//!
//! # Example
//!
//! ```no_run
//! use futures_util::stream::StreamExt;
//! use pubnub_hyper::runtime::tokio_global::TokioGlobal;
//! use pubnub_hyper::transport::hyper::Hyper;
//! use pubnub_hyper::{core::data::channel, core::json::object, Builder};
//!
//! # async {
//! let transport = Hyper::new()
//!     .publish_key("demo")
//!     .subscribe_key("demo")
//!     .build()?;
//! let mut pubnub = Builder::new()
//!     .transport(transport)
//!     .runtime(TokioGlobal)
//!     .build();
//!
//! let message = object! {
//!     "username" => "JoeBob",
//!     "content" => "Hello, world!",
//! };
//!
//! let channel_name: channel::Name = "my-channel".parse().unwrap();
//! let mut stream = pubnub.subscribe(channel_name.clone()).await;
//! let timetoken = pubnub.publish(channel_name, message.clone()).await?;
//!
//! let received = stream.next().await;
//! assert_eq!(received.unwrap().json, message);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # };
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]

/// Re-export core for ease of use.
pub mod core {
    pub use pubnub_core::*;
}

/// A sensible default variant of the tokio runtime.
pub use crate::runtime::tokio_global::TokioGlobal as DefaultRuntime;

/// A sensible default variant of the hyper runtime.
pub use crate::transport::hyper::Hyper as DefaultTransport;

pub use crate::core::Builder;
use crate::core::PubNub as CorePubNub;

/// PubNub client bound to hyper transport and tokio runtime.
pub type PubNub = CorePubNub<DefaultTransport, DefaultRuntime>;

pub mod runtime;
pub mod transport;

#[macro_use]
mod macros;
