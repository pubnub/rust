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
//! ```
//! use futures_util::stream::StreamExt;
//! use pubnub_hyper::{core::json::object, PubNub};
//!
//! # async {
//! let mut pubnub = PubNub::new("demo", "demo");
//!
//! let message = object!{
//!     "username" => "JoeBob",
//!     "content" => "Hello, world!",
//! };
//!
//! let mut stream = pubnub.subscribe("my-channel").await;
//! let timetoken = pubnub.publish("my-channel", message.clone()).await?;
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

/// Re-export core for ease of use.
pub mod core {
    pub use pubnub_core::*;
}

/// A sensible default variant of for tokio runtime.
pub use crate::runtime::tokio_global::TokioGlobal as DefaultRuntime;

/// A sensible default variant of the hyper runtime.
pub use crate::transport::hyper::Hyper as DefaultTransport;

use crate::core::{PubNub as Core, PubNubBuilder as CoreBuilder};

/// PubNub client bound with hyper transport and tokio runtime.
pub type PubNub = Core<DefaultTransport, DefaultRuntime>;

/// PubNubBuilder bound with hyper transport and tokio runtime.
pub type PubNubBuilder = CoreBuilder<DefaultTransport, DefaultRuntime>;

pub mod runtime;
pub mod transport;
