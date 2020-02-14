//! # PubNub Core
//!
//! Provides the common high-level logic for PubNub clients.
//!
//! - Fully `async`/`await` ready.
//! - Modular, bring your own [`Transport`] and [`Runtime`].
//! - Multiplexes subscription polling for multiple logical streams over a
//!   single transport invocation to optimize kernel network subsystem pressure.
//!
//! Build your own client, or use preconfigured [`pubnub-hyper`](pubnub_hyper).

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

pub use crate::message::{Instance, Message, Timetoken, Type};
pub use crate::pubnub::{PubNub, PubNubBuilder};
pub use crate::runtime::Runtime;
pub use crate::subscription::Subscription;
pub use crate::transport::Transport;
pub use json;

pub use async_trait::async_trait;

mod message;
mod pubnub;
mod runtime;
mod subscription;
mod transport;

#[cfg(test)]
mod tests;
