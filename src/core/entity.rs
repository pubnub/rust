//! # PubNub entity module
//!
//! This module contains the [`PubNubEntity`] trait, which is used to implement
//! a PubNub entity that can be used as a first-class citizen to access the
//! [`PubNub API`].
//!
//! [`PubNub API`]: https://www.pubnub.com/docs

use crate::{
    lib::{
        alloc::string::String,
        core::{
            cmp::PartialEq,
            fmt::{Debug, Formatter, Result},
        },
    },
    Channel, ChannelGroup, ChannelMetadata, UuidMetadata,
};

#[cfg(all(feature = "subscribe", feature = "std"))]
use crate::{
    core::{Deserializer, Transport},
    lib::alloc::{sync::Arc, vec::Vec},
    subscribe::{Subscribable, SubscribableType, Subscriber, Subscription, SubscriptionOptions},
};

pub(crate) trait PubNubEntity2 {
    /// Unique entity identifier.
    ///
    /// Identifier is important for the [`PubNub API`] and used as target
    /// identifier for used API.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    fn id(&self) -> String;
}

pub(crate) enum PubNubEntity<T, D> {
    Channel(Channel<T, D>),
    ChannelGroup(ChannelGroup<T, D>),
    ChannelMetadata(ChannelMetadata<T, D>),
    UuidMetadata(UuidMetadata<T, D>),
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> PubNubEntity<T, D> {
    pub(crate) fn names(&self, presence: bool) -> Vec<String> {
        match self {
            Self::Channel(channel) => channel.names(presence),
            Self::ChannelGroup(channel_group) => channel_group.names(presence),
            Self::ChannelMetadata(channel_metadata) => channel_metadata.names(presence),
            Self::UuidMetadata(uuid_metadata) => uuid_metadata.names(presence),
        }
    }

    pub(crate) fn r#type(&self) -> SubscribableType {
        match self {
            Self::Channel(channel) => channel.r#type(),
            Self::ChannelGroup(channel_group) => channel_group.r#type(),
            Self::ChannelMetadata(channel_metadata) => channel_metadata.r#type(),
            Self::UuidMetadata(uuid_metadata) => uuid_metadata.r#type(),
        }
    }

    /// Increase the subscriptions count.
    ///
    /// Increments the value of the subscriptions count by 1.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn increase_subscriptions_count(&self) {
        match self {
            Self::Channel(channel) => channel.increase_subscriptions_count(),
            Self::ChannelGroup(channel_group) => channel_group.increase_subscriptions_count(),
            Self::ChannelMetadata(channel_metadata) => {
                channel_metadata.increase_subscriptions_count()
            }
            Self::UuidMetadata(uuid_metadata) => uuid_metadata.increase_subscriptions_count(),
        }
    }

    /// Decrease the subscriptions count.
    ///
    /// Decrements the value of the subscriptions count by 1.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    ///
    /// > As long as entity used by at least one subscription it can't be
    /// > removed from subscription
    /// loop.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn decrease_subscriptions_count(&self) {
        match self {
            Self::Channel(channel) => channel.decrease_subscriptions_count(),
            Self::ChannelGroup(channel_group) => channel_group.decrease_subscriptions_count(),
            Self::ChannelMetadata(channel_metadata) => {
                channel_metadata.decrease_subscriptions_count()
            }
            Self::UuidMetadata(uuid_metadata) => uuid_metadata.decrease_subscriptions_count(),
        }
    }

    /// Current count of subscriptions.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    ///
    /// # Returns
    ///
    /// Returns the current count of subscriptions.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn subscriptions_count(&self) -> usize {
        match self {
            Self::Channel(channel) => channel.subscriptions_count(),
            Self::ChannelGroup(channel_group) => channel_group.subscriptions_count(),
            Self::ChannelMetadata(channel_metadata) => channel_metadata.subscriptions_count(),
            Self::UuidMetadata(uuid_metadata) => uuid_metadata.subscriptions_count(),
        }
    }
}

impl<T, D> Clone for PubNubEntity<T, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Channel(channel) => Self::Channel(channel.clone()),
            Self::ChannelGroup(channel_group) => Self::ChannelGroup(channel_group.clone()),
            Self::ChannelMetadata(channel_metadata) => {
                Self::ChannelMetadata(channel_metadata.clone())
            }
            Self::UuidMetadata(uuid_metadata) => Self::UuidMetadata(uuid_metadata.clone()),
        }
    }
}

impl<T, D> PartialEq for PubNubEntity<T, D> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Channel(channel_a) => {
                let Self::Channel(channel_b) = other else {
                    return false;
                };
                channel_a.eq(channel_b)
            }
            Self::ChannelGroup(channel_group_a) => {
                let Self::ChannelGroup(channel_group_b) = other else {
                    return false;
                };
                channel_group_a.eq(channel_group_b)
            }
            Self::ChannelMetadata(channel_metadata_a) => {
                let Self::ChannelMetadata(channel_metadata_b) = other else {
                    return false;
                };
                channel_metadata_a.eq(channel_metadata_b)
            }
            Self::UuidMetadata(uuid_metadata_a) => {
                let Self::UuidMetadata(uuid_metadata_b) = other else {
                    return false;
                };
                uuid_metadata_a.eq(uuid_metadata_b)
            }
        }
    }
}

impl<T, D> Debug for PubNubEntity<T, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Channel(channel) => write!(f, "Channel({channel:?})"),
            Self::ChannelGroup(channel_group) => write!(f, "ChannelGroup({channel_group:?})"),
            Self::ChannelMetadata(channel_metadata) => {
                write!(f, "ChannelMetadata({channel_metadata:?})")
            }
            Self::UuidMetadata(uuid_metadata) => write!(f, "UuidMetadata({uuid_metadata:?})"),
        }
    }
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> Subscriber<T, D> for PubNubEntity<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn subscription(&self, options: Option<Vec<SubscriptionOptions>>) -> Arc<Subscription<T, D>> {
        match self {
            PubNubEntity::Channel(channel) => channel.subscription(options),
            PubNubEntity::ChannelGroup(channel_group) => channel_group.subscription(options),
            PubNubEntity::ChannelMetadata(channel_metadata) => {
                channel_metadata.subscription(options)
            }
            PubNubEntity::UuidMetadata(uuid_metadata) => uuid_metadata.subscription(options),
        }
    }
}
