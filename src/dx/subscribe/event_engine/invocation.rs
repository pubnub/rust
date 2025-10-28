use crate::{
    core::event_engine::EffectInvocation,
    dx::subscribe::{
        event_engine::{SubscribeEffect, SubscribeEvent, SubscriptionInput},
        result::Update,
        ConnectionStatus, SubscriptionCursor,
    },
    lib::{
        alloc::vec::Vec,
        core::fmt::{Display, Formatter, Result},
    },
};

/// Subscribe effect invocations
///
/// Invocation is form of intention to call some action without any information
/// about its implementation.
#[derive(Debug)]
pub(crate) enum SubscribeEffectInvocation {
    /// Initial subscribe effect invocation.
    Handshake {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which will be source of
        /// real-time updates after initial subscription completion.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: Option<SubscriptionCursor>,
    },

    /// Cancel initial subscribe effect invocation.
    CancelHandshake,

    /// Receive updates effect invocation.
    Receive {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which real-time updates
        /// will be delivered.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscriptionCursor,
    },

    /// Cancel receive updates effect invocation.
    CancelReceive,

    /// Status change notification effect invocation.
    EmitStatus(ConnectionStatus),

    /// Received updates notification effect invocation.
    EmitMessages(Vec<Update>, SubscriptionCursor),

    /// Terminate Subscribe Event Engine processing loop.
    TerminateEventEngine,
}

impl EffectInvocation for SubscribeEffectInvocation {
    type Effect = SubscribeEffect;
    type Event = SubscribeEvent;

    fn id(&self) -> &str {
        match self {
            Self::Handshake { .. } => "HANDSHAKE",
            Self::CancelHandshake { .. } => "CANCEL_HANDSHAKE",
            Self::Receive { .. } => "RECEIVE_MESSAGES",
            Self::CancelReceive { .. } => "CANCEL_RECEIVE_MESSAGES",
            Self::EmitStatus(_) => "EMIT_STATUS",
            Self::EmitMessages(_, _) => "EMIT_MESSAGES",
            Self::TerminateEventEngine => "TERMINATE_EVENT_ENGINE",
        }
    }

    fn is_managed(&self) -> bool {
        matches!(self, Self::Handshake { .. } | Self::Receive { .. })
    }

    fn is_cancelling(&self) -> bool {
        matches!(
            self,
            Self::CancelHandshake { .. } | Self::CancelReceive { .. }
        )
    }

    fn cancelling_effect(&self, effect: &Self::Effect) -> bool {
        (matches!(effect, SubscribeEffect::Handshake { .. })
            && matches!(self, Self::CancelHandshake { .. }))
            || (matches!(effect, SubscribeEffect::Receive { .. })
                && matches!(self, Self::CancelReceive { .. }))
    }

    fn is_terminating(&self) -> bool {
        matches!(self, Self::TerminateEventEngine)
    }
}

impl Display for SubscribeEffectInvocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Handshake { .. } => write!(f, "HANDSHAKE"),
            Self::CancelHandshake => write!(f, "CANCEL_HANDSHAKE"),
            Self::Receive { .. } => write!(f, "RECEIVE_MESSAGES"),
            Self::CancelReceive { .. } => write!(f, "CANCEL_RECEIVE_MESSAGES"),
            Self::EmitStatus(status) => write!(f, "EMIT_STATUS({status:?})"),
            Self::EmitMessages(messages, _) => write!(f, "EMIT_MESSAGES({messages:?})"),
            Self::TerminateEventEngine => write!(f, "TERMINATE_EVENT_ENGINE"),
        }
    }
}
