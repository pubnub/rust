//! Heartbeat Event Engine event module.
//!
//! The module contains the [`PresenceEvent`] type, which describes available
//! event engine transition events.

use crate::core::{event_engine::Event, PubNubError};

#[derive(Debug)]
pub(crate) enum PresenceEvent {
    /// Announce join to channels and groups.
    ///
    /// Announce `user_id` presence on new channels and groups.
    Joined {
        /// `user_id` presence announcement interval.
        heartbeat_interval: u64,

        /// Optional list of channels.
        ///
        /// List of channels for which `user_id` presence should be announced.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which `user_id` presence should be
        /// announced.
        channel_groups: Option<Vec<String>>,
    },

    /// Announce leave on channels and groups.
    ///
    /// Announce `user_id` leave from channels and groups.
    Left {
        /// Whether `user_id` leave should be announced or not.
        ///
        /// When set to `true` and `user_id` will unsubscribe, the client
        /// wouldn't announce `leave`, and as a result, there will be no
        /// `leave` presence event generated.
        suppress_leave_events: bool,

        /// Optional list of channels.
        ///
        /// List of channels for which `user_id` should leave.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which `user_id` should leave.
        channel_groups: Option<Vec<String>>,
    },

    /// Announce leave on all channels and groups.
    ///
    /// Announce `user_id` leave from all channels and groups.
    LeftAll {
        /// Whether `user_id` leave should be announced or not.
        ///
        /// When set to `true` and `user_id` will unsubscribe, the client
        /// wouldn't announce `leave`, and as a result, there will be no
        /// `leave` presence event generated.
        suppress_leave_events: bool,
    },

    /// Heartbeat completed successfully.
    ///
    /// Emitted when [`PubNub`] network returned `OK` response.
    HeartbeatSuccess,

    /// Heartbeat completed with an error.
    ///
    /// Emitted when another heartbeat effect attempt was unable to receive
    /// response from [`PubNub`] network (network or permission issues).
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    HeartbeatFailure { reason: PubNubError },

    /// Restore heartbeating.
    ///
    /// Re-launch heartbeat event engine.
    Reconnect,

    /// Temporarily stop event engine.
    ///
    /// Suspend any delayed and waiting heartbeat endpoint calls till
    /// `Reconnect` event will be triggered again.
    Disconnect,

    /// Delay times up event.
    ///
    /// Emitted when `delay` reaches the end and should transit to the next
    /// state.
    TimesUp,
}

impl Event for PresenceEvent {
    fn id(&self) -> &str {
        match self {
            Self::Joined { .. } => "JOINED",
            Self::Left { .. } => "LEFT",
            Self::LeftAll { .. } => "LEFT_ALL",
            Self::HeartbeatSuccess => "HEARTBEAT_SUCCESS",
            Self::HeartbeatFailure { .. } => "HEARTBEAT_FAILURE",
            Self::Reconnect => "RECONNECT",
            Self::Disconnect => "DISCONNECT",
            Self::TimesUp => "TIMES_UP",
        }
    }
}
