//! # PubNub Core
//!
//! Provides the common high-level logic for PubNub clients.
//!
//! - Fully `async`/`await` ready.
//! - Modular, bring your own [`Transport`] and [`Runtime`].
//! - Multiplexes subscription polling for multiple logical streams over a
//!   single transport invocation to optimize kernel network subsystem pressure.
//!
//! Build your own client, or use preconfigured [`pubnub-hyper`](pubnub-hyper).

#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    broken_intra_doc_links
)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

pub use crate::builder::Builder;
pub use crate::pubnub::PubNub;
pub use crate::runtime::Runtime;
pub use crate::subscription::Subscription;
pub use crate::transport::{Service as TransportService, Transport};
pub use json;

pub use async_trait::async_trait;

mod builder;
pub mod data;
mod pubnub;
mod runtime;
mod subscription;
mod transport;

#[cfg(feature = "mock")]
pub mod mock;
