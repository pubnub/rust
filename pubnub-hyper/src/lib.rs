//! # PubNub Client SDK for Rust coupled with Hyper
//!
//! - Fully `async`/`await` ready.
//! - Uses Tokio and Hyper to provide an ultra-fast, incredibly reliable message transport over the
//!   PubNub edge network.
//! - Optimizes for minimal network sockets with an infinite number of logical streams.
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

pub use crate::runtime::tokio_global::Runtime;
pub use crate::transport::hyper::Transport;

use crate::core::{PubNub as Core, PubNubBuilder as CoreBuilder};

/// PubNub client bound with hyper transport and tokio runtime.
pub type PubNub = Core<Transport, Runtime>;

/// PubNubBuilder bound with hyper transport and tokio runtime.
pub type PubNubBuilder = CoreBuilder<Transport, Runtime>;

pub mod runtime;
pub mod transport;
