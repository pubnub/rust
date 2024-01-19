//! # Subscribable module.
//!
//! This module contains the [`Subscribable`] trait, which is used to implement
//! objects that can deliver real-time updates from the [`PubNub`] network.
//!
//! [`PubNub`]: https://www.pubnub.com

use crate::{
    dx::pubnub_client::PubNubClientInstance,
    lib::alloc::{string::String, sync::Weak, vec::Vec},
};

/// Types of subscribable objects.
///
/// Subscribable can be separated by their place in subscribe REST API:
/// * `URI path` - channel-like objects which represent single entity
///   ([`Channel`], [`ChannelMetadata`], [`UuidMetadata`])
/// * `query parameter` - entities which represent group of entities
///   ([`ChannelGroup`])
pub enum SubscribableType {
    /// Channel identifier, which is part of the URI path.
    Channel,

    /// Channel group identifiers, which is part of the query parameters.
    ChannelGroup,
}

/// Subscribable entities' trait.
///
/// Only entities that implement this trait can subscribe to their real-time
/// events.
pub trait Subscribable<T, D> {
    /// Names for object to be used in subscription.
    ///
    /// Provided strings will be used with multiplexed subscribe REST API call.
    fn names(&self, presence: bool) -> Vec<String>;

    /// Type of subscription object.
    fn r#type(&self) -> SubscribableType;

    /// PubNub client instance which created entity.
    fn client(&self) -> Weak<PubNubClientInstance<T, D>>;
}
