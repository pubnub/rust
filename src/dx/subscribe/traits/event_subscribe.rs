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
    /// Use the receiver to subscribe for real-time updates.
    fn subscribe(&self);

    /// Use the receiver to subscribe for real-time updates starting at a
    /// specific time.
    ///
    /// # Arguments
    ///
    /// - `cursor` - `SubscriptionCursor` from which, the client should try to
    ///   catch up on real-time events. `cursor` also can be provided as `usize`
    ///   or `String` with a 17-digit PubNub  timetoken. If `cursor` doesn't
    ///   satisfy the requirements, it will be ignored.
    fn subscribe_with_timetoken<SC>(&self, cursor: SC)
    where
        SC: Into<SubscriptionCursor>;

    /// Use receiver to stop receiving real-time updates.
    fn unsubscribe(&self);
}
