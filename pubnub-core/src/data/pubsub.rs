//! Publish/subscribe API related types.

use super::channel;

/// A subscription destination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscribeTo {
    /// Subscribe to Channel by name.
    Channel(channel::Name),
    /// Subscribe to a set of Channels using a wildcard specifier.
    ChannelWildcard(channel::WildcardSpec),
    /// Subscribe to a Channel Group by name.
    ChannelGroup(channel::Name),
}

impl SubscribeTo {
    /// Returns a channel name if this is a [`SubscribeTo::Channel`].
    #[must_use]
    pub fn as_channel(&self) -> Option<&channel::Name> {
        match self {
            SubscribeTo::Channel(ref v) => Some(v),
            _ => None,
        }
    }

    /// Returns a channel wildcard spec this is a [`SubscribeTo::ChannelWildcard`].
    #[must_use]
    pub fn as_channel_wildcard(&self) -> Option<&channel::WildcardSpec> {
        match self {
            SubscribeTo::ChannelWildcard(ref v) => Some(v),
            _ => None,
        }
    }

    /// Returns a channel group this is a [`SubscribeTo::ChannelGroup`].
    #[must_use]
    pub fn as_channel_group(&self) -> Option<&channel::Name> {
        match self {
            SubscribeTo::ChannelGroup(ref v) => Some(v),
            _ => None,
        }
    }
}
