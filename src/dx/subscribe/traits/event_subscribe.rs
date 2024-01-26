//! # Event subscriber module.
//!
//! This module contains the [`EventSubscriber`] trait, which is used to
//! implement objects that provides functionality to subscribe and unsubscribe
//! from real-time events stream.

use crate::subscribe::SubscriptionCursor;

/// Subscriber trait.
///
/// Types that implement this trait can change activity of real-time events
/// processing for specific or set of entities.
pub trait EventSubscriber {
    /// Use receiver to subscribe for real-time updates.
    fn subscribe(&self, cursor: Option<SubscriptionCursor>);

    /// Use receiver to stop receiving real-time updates.
    fn unsubscribe(&self);
}
