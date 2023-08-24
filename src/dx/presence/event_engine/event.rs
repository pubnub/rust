//! Heartbeat Event Engine event module.
//!
//! The module contains the [`PresenceEvent`] type, which describes available
//! event engine transition events.

use crate::core::{event_engine::Event, PubNubError};

pub(crate) enum PresenceEvent {
    /// Announce join to channels and groups.
    ///
    /// Announce `user_id` presence on new channels and groups.
    #[allow(dead_code)]
    Joined {
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
    #[allow(dead_code)]
    Left {
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
    #[allow(dead_code)]
    LeftAll,

    /// Heartbeat completed successfully.
    ///
    /// Emitted when [`PubNub`] network returned `OK` response.
    #[allow(dead_code)]
    HeartbeatSuccess,

    /// Heartbeat completed with an error.
    ///
    /// Emitted when another heartbeat effect attempt was unable to receive
    /// response from [`PubNub`] network (network or permission issues).
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    #[allow(dead_code)]
    HeartbeatFailure { reason: PubNubError },

    /// All heartbeat attempts was unsuccessful.
    ///
    /// Emitted when heartbeat attempts reached maximum allowed count (according
    /// to retry / reconnection policy) and all following attempts should be
    /// stopped.
    #[allow(dead_code)]
    HeartbeatGiveUp { reason: PubNubError },

    /// Restore heartbeating.
    ///
    /// Re-launch heartbeat event engine.
    #[allow(dead_code)]
    Reconnect,

    /// Temporarily stop event engine.
    ///
    /// Suspend any delayed and waiting heartbeat endpoint calls till
    /// `Reconnect` event will be triggered again.
    #[allow(dead_code)]
    Disconnect,

    /// Delay times up event.
    ///
    /// Emitted when `delay` reaches the end and should transit to the next
    /// state.
    #[allow(dead_code)]
    TimesUp,
}

impl Event for PresenceEvent {
    fn id(&self) -> &str {
        match self {
            Self::Joined { .. } => "JOINED",
            Self::Left { .. } => "LEFT",
            Self::LeftAll => "LEFT_ALL",
            Self::HeartbeatSuccess => "HEARTBEAT_SUCCESS",
            Self::HeartbeatFailure { .. } => "HEARTBEAT_FAILED",
            Self::HeartbeatGiveUp { .. } => "HEARTBEAT_GIVEUP",
            Self::Reconnect => "RECONNECT",
            Self::Disconnect => "DISCONNECT",
            Self::TimesUp => "TIMES_UP",
        }
    }
}
