//! Presence event engine module types.
//!
//! This module contains the [`PresenceInput`] type, which represents
//! user-provided channels and groups for which `user_id` presence should be
//! managed.

use crate::lib::{
    alloc::{collections::HashSet, string::String, vec::Vec},
    core::ops::{Add, Sub},
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

        let channel_groups_is_empty = channel_groups.as_ref().is_none_or(|set| set.is_empty());
        let channels_is_empty = channels.as_ref().is_none_or(|set| set.is_empty());

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

    fn join_sets(
        &self,
        lhs: &Option<HashSet<String>>,
        rhs: &Option<HashSet<String>>,
    ) -> Option<HashSet<String>> {
        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => Some(lhs.iter().cloned().chain(rhs.to_owned()).collect()),
            (Some(lhs), None) => Some(lhs.to_owned()),
            (None, Some(rhs)) => Some(rhs.to_owned()),
            _ => None,
        }
    }

    fn sub_sets(
        &self,
        lhs: &Option<HashSet<String>>,
        rhs: &Option<HashSet<String>>,
    ) -> Option<HashSet<String>> {
        match (lhs.to_owned(), rhs.to_owned()) {
            (Some(lhs), Some(rhs)) => Some(&lhs - &rhs).filter(|diff| !diff.is_empty()),
            (Some(lhs), None) => Some(lhs),
            _ => None,
        }
    }
}

impl Add for PresenceInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let channel_groups = self.join_sets(&self.channel_groups, &rhs.channel_groups);
        let channels = self.join_sets(&self.channels, &rhs.channels);
        let channel_groups_is_empty = channel_groups.as_ref().is_none_or(|set| set.is_empty());
        let channels_is_empty = channels.as_ref().is_none_or(|set| set.is_empty());

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
        let channel_groups = self.sub_sets(&self.channel_groups, &rhs.channel_groups);
        let channels = self.sub_sets(&self.channels, &rhs.channels);
        let channel_groups_is_empty = channel_groups.as_ref().is_none_or(|set| set.is_empty());
        let channels_is_empty = channels.as_ref().is_none_or(|set| set.is_empty());

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
pub struct PresenceParameters<'execution> {
    /// List of channel for which `user_id` presence should be announced.
    pub channels: &'execution Option<Vec<String>>,

    /// List of channel groups for which `user_id` presence should be announced.
    pub channel_groups: &'execution Option<Vec<String>>,
}

#[cfg(test)]
mod it_should {
    use super::*;

    #[test]
    fn create_empty_input() {
        let input = PresenceInput::new(&None, &None);
        assert!(input.is_empty);
    }

    #[test]
    fn create_input_with_unique_channels() {
        let input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-1".into(),
            ]),
            &None,
        );

        assert!(!input.is_empty);
        assert_eq!(input.channels().unwrap().len(), 2);
        assert_eq!(
            input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec!["channel-1".to_string(), "channel-2".to_string()]
        );
    }

    #[test]
    fn create_input_with_unique_channel_groups() {
        let input = PresenceInput::new(
            &None,
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-2".into(),
            ]),
        );

        assert!(!input.is_empty);
        assert_eq!(input.channel_groups().unwrap().len(), 2);
        assert_eq!(
            input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec!["channel-group-1".to_string(), "channel-group-2".to_string()]
        );
    }

    #[test]
    fn add_unique_channels_to_empty_input() {
        let empty_input = PresenceInput::new(&None, &None);
        let input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-1".into(),
            ]),
            &None,
        );

        assert!(!input.is_empty);

        let joint_input = empty_input + input;

        assert!(!joint_input.is_empty);
        assert_eq!(joint_input.channels().unwrap().len(), 2);
        assert_eq!(
            joint_input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec!["channel-1".to_string(), "channel-2".to_string()]
        );
        assert!(joint_input.channel_groups().is_none());
    }

    #[test]
    fn add_unique_channel_groups_to_empty_input() {
        let empty_input = PresenceInput::new(&None, &None);
        let input = PresenceInput::new(
            &None,
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-2".into(),
            ]),
        );

        assert!(!input.is_empty);

        let joint_input = empty_input + input;

        assert!(!joint_input.is_empty);
        assert!(joint_input.channels().is_none());
        assert_eq!(joint_input.channel_groups().unwrap().len(), 2);
        assert_eq!(
            joint_input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec!["channel-group-1".to_string(), "channel-group-2".to_string()]
        );
    }

    #[test]
    fn add_unique_channels_and_channel_groups_to_existing_input() {
        let existing_input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-4".into(),
                "channel-2".into(),
            ]),
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-3".into(),
                "channel-group-5".into(),
            ]),
        );
        let input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-1".into(),
            ]),
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-2".into(),
            ]),
        );

        assert!(!existing_input.is_empty);
        assert!(!input.is_empty);

        let joint_input = existing_input + input;

        assert!(!joint_input.is_empty);
        assert_eq!(joint_input.channels().unwrap().len(), 3);
        assert_eq!(joint_input.channel_groups().unwrap().len(), 4);
        assert_eq!(
            joint_input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec![
                "channel-1".to_string(),
                "channel-2".to_string(),
                "channel-4".to_string()
            ]
        );
        assert_eq!(
            joint_input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec![
                "channel-group-1".to_string(),
                "channel-group-2".to_string(),
                "channel-group-3".to_string(),
                "channel-group-5".to_string()
            ]
        );
    }

    #[test]
    fn remove_channels_from_empty_input() {
        let empty_input = PresenceInput::new(&None, &None);
        let input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-1".into(),
            ]),
            &None,
        );

        assert!(!input.is_empty);

        let diff_input = empty_input - input;

        assert!(diff_input.is_empty);
        assert!(diff_input.channels().is_none());
        assert!(diff_input.channel_groups().is_none());
    }

    #[test]
    fn remove_channel_groups_from_empty_input() {
        let empty_input = PresenceInput::new(&None, &None);
        let input = PresenceInput::new(
            &None,
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-1".into(),
            ]),
        );

        assert!(!input.is_empty);

        let diff_input = empty_input - input;

        assert!(diff_input.is_empty);
        assert!(diff_input.channels().is_none());
        assert!(diff_input.channel_groups().is_none());
    }

    #[test]
    fn remove_unique_channels_from_existing_input() {
        let existing_input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-3".into(),
            ]),
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-3".into(),
            ]),
        );
        let input = PresenceInput::new(&Some(vec!["channel-2".into(), "channel-2".into()]), &None);

        assert!(!existing_input.is_empty);
        assert!(!input.is_empty);

        let diff_input = existing_input - input;

        assert!(!diff_input.is_empty);
        assert_eq!(diff_input.channels().unwrap().len(), 2);
        assert_eq!(diff_input.channel_groups().unwrap().len(), 3);
        assert_eq!(
            diff_input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec!["channel-1".to_string(), "channel-3".to_string()]
        );
        assert_eq!(
            diff_input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec![
                "channel-group-1".to_string(),
                "channel-group-2".to_string(),
                "channel-group-3".to_string(),
            ]
        );
    }

    #[test]
    fn remove_unique_channel_groups_from_existing_input() {
        let existing_input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-3".into(),
            ]),
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-3".into(),
            ]),
        );
        let input = PresenceInput::new(&None, &Some(vec!["channel-group-1".into()]));

        assert!(!existing_input.is_empty);
        assert!(!input.is_empty);

        let diff_input = existing_input - input;

        assert!(!diff_input.is_empty);
        assert_eq!(diff_input.channels().unwrap().len(), 3);
        assert_eq!(diff_input.channel_groups().unwrap().len(), 2);
        assert_eq!(
            diff_input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec![
                "channel-1".to_string(),
                "channel-2".to_string(),
                "channel-3".to_string()
            ]
        );
        assert_eq!(
            diff_input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec!["channel-group-2".to_string(), "channel-group-3".to_string(),]
        );
    }

    #[test]
    fn remove_unique_channels_and_channel_groups_from_existing_input() {
        let existing_input = PresenceInput::new(
            &Some(vec![
                "channel-1".into(),
                "channel-2".into(),
                "channel-3".into(),
            ]),
            &Some(vec![
                "channel-group-1".into(),
                "channel-group-2".into(),
                "channel-group-3".into(),
            ]),
        );
        let input = PresenceInput::new(
            &Some(vec!["channel-3".into()]),
            &Some(vec!["channel-group-2".into(), "channel-group-3".into()]),
        );

        assert!(!existing_input.is_empty);
        assert!(!input.is_empty);

        let diff_input = existing_input - input;

        assert!(!diff_input.is_empty);
        assert_eq!(diff_input.channels().unwrap().len(), 2);
        assert_eq!(diff_input.channel_groups().unwrap().len(), 1);
        assert_eq!(
            diff_input
                .channels()
                .map(|mut channels| {
                    channels.sort();
                    channels
                })
                .unwrap(),
            vec!["channel-1".to_string(), "channel-2".to_string(),]
        );
        assert_eq!(
            diff_input
                .channel_groups()
                .map(|mut groups| {
                    groups.sort();
                    groups
                })
                .unwrap(),
            vec!["channel-group-1".to_string(),]
        );
    }

    #[test]
    fn remove_all_channels_and_channel_groups_from_existing_input() {
        let existing_input = PresenceInput::new(
            &Some(vec!["channel-1".into(), "channel-2".into()]),
            &Some(vec!["channel-group-1".into(), "channel-group-2".into()]),
        );
        let input = PresenceInput::new(
            &Some(vec!["channel-1".into(), "channel-2".into()]),
            &Some(vec!["channel-group-1".into(), "channel-group-2".into()]),
        );

        assert!(!existing_input.is_empty);
        assert!(!input.is_empty);

        let diff_input = existing_input - input;

        assert!(diff_input.is_empty);
        assert!(diff_input.channels().is_none());
        assert!(diff_input.channel_groups().is_none());
    }
}
