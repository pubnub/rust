//! Heartbeat Event Engine effect invocation module.
//!
//! The module contains the [`PresenceEffectInvocation`] type, which describes
//! available event engine effect invocations.

use crate::{
    core::{event_engine::EffectInvocation, PubNubError},
    lib::core::fmt::{Display, Formatter, Result},
    presence::event_engine::{PresenceEffect, PresenceEvent, PresenceInput},
};

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum PresenceEffectInvocation {
    /// Heartbeat effect invocation.
    Heartbeat {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,
    },

    /// Delayed heartbeat effect invocation.
    DelayedHeartbeat {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,

        /// Delayed heartbeat retry attempt.
        ///
        /// Used to track overall number of delayed heartbeat retry attempts.
        attempts: u8,

        /// Delayed heartbeat attempt failure reason.
        reason: PubNubError,
    },

    /// Cancel delayed heartbeat effect invocation.
    CancelDelayedHeartbeat,

    /// Leave effect invocation.
    Leave {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// should leave.
        input: PresenceInput,
    },

    /// Delay effect invocation.
    Wait {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,
    },

    /// Cancel delay effect invocation.
    CancelWait,

    /// Terminate Presence Event Engine processing loop.
    TerminateEventEngine,
}

impl EffectInvocation for PresenceEffectInvocation {
    type Effect = PresenceEffect;
    type Event = PresenceEvent;

    fn id(&self) -> &str {
        match self {
            Self::Heartbeat { .. } => "HEARTBEAT",
            Self::DelayedHeartbeat { .. } => "DELAYED_HEARTBEAT",
            Self::CancelDelayedHeartbeat => "CANCEL_DELAYED_HEARTBEAT",
            Self::Leave { .. } => "LEAVE",
            Self::Wait { .. } => "WAIT",
            Self::CancelWait => "CANCEL_WAIT",
            Self::TerminateEventEngine => "TERMINATE_EVENT_ENGINE",
        }
    }

    fn is_managed(&self) -> bool {
        matches!(self, Self::Wait { .. } | Self::DelayedHeartbeat { .. })
    }

    fn is_cancelling(&self) -> bool {
        matches!(self, Self::CancelDelayedHeartbeat | Self::CancelWait)
    }

    fn cancelling_effect(&self, effect: &Self::Effect) -> bool {
        (matches!(effect, PresenceEffect::DelayedHeartbeat { .. })
            && matches!(self, Self::CancelDelayedHeartbeat { .. }))
            || (matches!(effect, PresenceEffect::Wait { .. })
                && matches!(self, Self::CancelWait { .. }))
    }

    fn is_terminating(&self) -> bool {
        matches!(self, Self::TerminateEventEngine)
    }
}

impl Display for PresenceEffectInvocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Heartbeat { .. } => write!(f, "HEARTBEAT"),
            Self::DelayedHeartbeat { .. } => write!(f, "DELAYED_HEARTBEAT"),
            Self::CancelDelayedHeartbeat => write!(f, "CANCEL_DELAYED_HEARTBEAT"),
            Self::Leave { .. } => write!(f, "LEAVE"),
            Self::Wait { .. } => write!(f, "WAIT"),
            Self::CancelWait => write!(f, "CANCEL_WAIT"),
            Self::TerminateEventEngine => write!(f, "TERMINATE_EVENT_ENGINE"),
        }
    }
}
