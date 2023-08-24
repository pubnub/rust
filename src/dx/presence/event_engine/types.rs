//! Presence event engine module types.
//!
//! This module contains the [`PresenceInput`] type, which represents
//! user-provided channels and groups for which `user_id` presence should be
//! managed.

use crate::{
    core::PubNubError,
    lib::{
        alloc::{collections::HashSet, string::String, vec::Vec},
        core::ops::{Add, Sub},
    },
};

/// User-provided channels and groups for presence.
///
/// Object contains information about channels and groups which should be used
/// with presence event engine states.
#[derive(Clone, Debug, PartialEq)]
pub struct PresenceInput {
    /// Optional list of channels.
    ///
    /// List of channels for which `user_id` presence should be managed.
    pub channels: Option<HashSet<String>>,

    /// Optional list of channel groups.
    ///
    /// List of channel groups for which `user_id` presence should be managed.
    pub channel_groups: Option<HashSet<String>>,

    /// Whether user input is empty or not.
    pub is_empty: bool,
}

impl PresenceInput {
    pub fn new(channels: &Option<Vec<String>>, channel_groups: &Option<Vec<String>>) -> Self {
        let channels = channels.as_ref().map(|channels| {
            channels.iter().fold(HashSet::new(), |mut acc, channel| {
                acc.insert(channel.clone());
                acc
            })
        });
        let channel_groups = channel_groups.as_ref().map(|groups| {
            groups.iter().fold(HashSet::new(), |mut acc, group| {
                acc.insert(group.clone());
                acc
            })
        });

        let channel_groups_is_empty = channel_groups.as_ref().map_or(true, |set| set.is_empty());
        let channels_is_empty = channels.as_ref().map_or(true, |set| set.is_empty());

        Self {
            channels,
            channel_groups,
            is_empty: channel_groups_is_empty && channels_is_empty,
        }
    }

    pub fn channels(&self) -> Option<Vec<String>> {
        self.channels.clone().map(|ch| ch.into_iter().collect())
    }

    pub fn channel_groups(&self) -> Option<Vec<String>> {
        self.channel_groups
            .clone()
            .map(|ch| ch.into_iter().collect())
    }
}

impl Add for PresenceInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let channel_groups: Option<HashSet<String>> =
            match (self.channel_groups, rhs.channel_groups) {
                (Some(lhs), Some(rhs)) => Some(lhs.into_iter().chain(rhs).collect()),
                (Some(lhs), None) => Some(lhs),
                (None, Some(rhs)) => Some(rhs),
                _ => None,
            };
        let channels: Option<HashSet<String>> = match (self.channels, rhs.channels) {
            (Some(lhs), Some(rhs)) => Some(lhs.into_iter().chain(rhs).collect()),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            _ => None,
        };

        let channel_groups_is_empty = channel_groups.as_ref().map_or(true, |set| set.is_empty());
        let channels_is_empty = channels.as_ref().map_or(true, |set| set.is_empty());

        Self {
            channels,
            channel_groups,
            is_empty: channel_groups_is_empty && channels_is_empty,
        }
    }
}

impl Sub for PresenceInput {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let channel_groups: Option<HashSet<String>> =
            match (self.channel_groups, rhs.channel_groups) {
                (Some(lhs), Some(rhs)) => Some(&lhs - &rhs),
                (Some(lhs), None) => Some(lhs),
                _ => None,
            };
        let channels: Option<HashSet<String>> = match (self.channels, rhs.channels) {
            (Some(lhs), Some(rhs)) => Some(&lhs - &rhs),
            (Some(lhs), None) => Some(lhs),
            _ => None,
        };

        let channel_groups_is_empty = channel_groups.as_ref().map_or(true, |set| set.is_empty());
        let channels_is_empty = channels.as_ref().map_or(true, |set| set.is_empty());

        Self {
            channels,
            channel_groups,
            is_empty: channel_groups_is_empty && channels_is_empty,
        }
    }
}

#[cfg(feature = "std")]
#[derive(Clone)]
/// Presence event engine data.
///
/// Data objects are used by the presence event engine to communicate between
/// components.
pub(crate) struct PresenceParameters<'execution> {
    /// List of channel for which `user_id` presence should be announced.
    pub channels: &'execution Option<Vec<String>>,

    /// List of channel groups for which `user_id` presence should be announced.
    pub channel_groups: &'execution Option<Vec<String>>,

    /// How many consequent retry attempts has been made.
    pub attempt: u8,

    /// Reason why previous request created by presence event engine failed.
    pub reason: Option<PubNubError>,

    /// Effect identifier.
    ///
    /// Identifier of effect which requested to create request.
    pub effect_id: &'execution str,
}
