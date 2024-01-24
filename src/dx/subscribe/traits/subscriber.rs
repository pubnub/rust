//! # Events subscriber module.
//!
//! This module contains the [`Subscriber`] and [`MultiplexSubscriber`] traits
//! which is used by types to provide ability to subscribe for real-time events.

use crate::{
    lib::alloc::vec::Vec,
    subscribe::{Subscription, SubscriptionOptions},
};

/// Trait representing a subscriber.
pub trait Subscriber<T: Send + Sync, D: Send + Sync> {
    /// Creates a new subscription with the specified options.
    ///
    /// # Arguments
    ///
    /// * `options` - The subscription options. Pass `None` if no specific
    ///   options should be applied.
    ///
    /// # Returns
    ///
    /// A [`Subscription`] object representing the newly created subscription to
    /// receiver's data stream events.
    fn subscription(&self, options: Option<Vec<SubscriptionOptions>>) -> Subscription<T, D>;
}
