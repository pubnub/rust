//! Heartbeat Event Engine effect invocation module.
//!
//! The module contains the [`PresenceEffectInvocation`] type, which describes
//! available event engine effect invocations.

use crate::{
    core::event_engine::EffectInvocation,
    lib::core::fmt::{Display, Formatter, Result},
    presence::event_engine::{PresenceEffect, PresenceEvent, PresenceInput},
};

#[derive(Debug)]
pub(crate) enum PresenceEffectInvocation {
    /// Heartbeat effect invocation.
    Heartbeat {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,
    },

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
            Self::Leave { .. } => "LEAVE",
            Self::Wait { .. } => "WAIT",
            Self::CancelWait => "CANCEL_WAIT",
            Self::TerminateEventEngine => "TERMINATE_EVENT_ENGINE",
        }
    }

    fn is_managed(&self) -> bool {
        matches!(self, Self::Wait { .. })
    }

    fn is_cancelling(&self) -> bool {
        matches!(self, Self::CancelWait)
    }

    fn cancelling_effect(&self, effect: &Self::Effect) -> bool {
        matches!(effect, PresenceEffect::Wait { .. }) && matches!(self, Self::CancelWait { .. })
    }

    fn is_terminating(&self) -> bool {
        matches!(self, Self::TerminateEventEngine)
    }
}

impl Display for PresenceEffectInvocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Heartbeat { .. } => write!(f, "HEARTBEAT"),
            Self::Leave { .. } => write!(f, "LEAVE"),
            Self::Wait { .. } => write!(f, "WAIT"),
            Self::CancelWait => write!(f, "CANCEL_WAIT"),
            Self::TerminateEventEngine => write!(f, "TERMINATE_EVENT_ENGINE"),
        }
    }
}
