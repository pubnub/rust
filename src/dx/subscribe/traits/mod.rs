//! # Subscription traits module
//!
//! This module provides set of traits which is implemented by types to support
//! `subscribe` and `presence` features.

#[doc(inline)]
pub use event_subscribe::EventSubscriber;
mod event_subscribe;

#[doc(inline)]
pub use subscriber::Subscriber;
mod subscriber;

#[doc(inline)]
pub use subscribable::{Subscribable, SubscribableType};
mod subscribable;

#[doc(inline)]
pub use event_emitter::EventEmitter;
mod event_emitter;

#[doc(inline)]
pub(super) use event_handler::EventHandler;
mod event_handler;
