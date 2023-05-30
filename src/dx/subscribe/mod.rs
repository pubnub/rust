//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

pub(crate) mod event_engine;

#[doc(inline)]
pub use types::{SubscribeCursor, SubscribeStatus};

pub mod types;

#[allow(dead_code)]
pub(crate) mod subscription;
