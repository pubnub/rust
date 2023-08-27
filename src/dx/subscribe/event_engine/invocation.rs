use crate::{
    core::{event_engine::EffectInvocation, PubNubError},
    dx::subscribe::{
        event_engine::{SubscribeEffect, SubscribeEvent, SubscribeInput},
        result::Update,
        SubscribeCursor, SubscribeStatus,
    },
    lib::{
        alloc::vec::Vec,
        core::fmt::{Display, Formatter, Result},
    },
};

/// Subscribe effect invocations
///
/// Invocation is form of intention to call some action without any information
/// about it's implementation.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum SubscribeEffectInvocation {
    /// Initial subscribe effect invocation.
    Handshake {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which will be source of
        /// real-time updates after initial subscription completion.
        input: SubscribeInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: Option<SubscribeCursor>,
    },

    /// Cancel initial subscribe effect invocation.
    CancelHandshake,

    /// Retry initial subscribe effect invocation.
    HandshakeReconnect {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which has been used
        /// during recently failed initial subscription.
        input: SubscribeInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: Option<SubscribeCursor>,

        /// Current initial subscribe retry attempt.
        ///
        /// Used to track overall number of initial subscription retry attempts.
        attempts: u8,

        /// Initial subscribe attempt failure reason.
        reason: PubNubError,
    },

    /// Cancel initial subscribe retry effect invocation.
    CancelHandshakeReconnect,

    /// Receive updates effect invocation.
    Receive {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which real-time updates
        /// will be delivered.
        input: SubscribeInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,
    },

    /// Cancel receive updates effect invocation.
    CancelReceive,

    /// Retry receive updates effect invocation.
    ReceiveReconnect {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups which has been used
        /// during recently failed receive updates.
        input: SubscribeInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,

        /// Current receive retry attempt.
        ///
        /// Used to track overall number of receive updates retry attempts.
        attempts: u8,

        /// Receive updates attempt failure reason.
        reason: PubNubError,
    },

    /// Cancel receive updates retry effect invocation.
    CancelReceiveReconnect,

    /// Status change notification effect invocation.
    EmitStatus(SubscribeStatus),

    /// Received updates notification effect invocation.
    EmitMessages(Vec<Update>),
}

impl EffectInvocation for SubscribeEffectInvocation {
    type Effect = SubscribeEffect;
    type Event = SubscribeEvent;

    fn id(&self) -> &str {
        match self {
            Self::Handshake { .. } => "HANDSHAKE",
            Self::CancelHandshake => "CANCEL_HANDSHAKE",
            Self::HandshakeReconnect { .. } => "HANDSHAKE_RECONNECT",
            Self::CancelHandshakeReconnect => "CANCEL_HANDSHAKE_RECONNECT",
            Self::Receive { .. } => "RECEIVE_MESSAGES",
            Self::CancelReceive { .. } => "CANCEL_RECEIVE_MESSAGES",
            Self::ReceiveReconnect { .. } => "RECEIVE_RECONNECT",
            Self::CancelReceiveReconnect { .. } => "CANCEL_RECEIVE_RECONNECT",
            Self::EmitStatus(_status) => "EMIT_STATUS",
            Self::EmitMessages(_messages) => "EMIT_MESSAGES",
        }
    }

    fn managed(&self) -> bool {
        matches!(
            self,
            Self::Handshake { .. }
                | Self::HandshakeReconnect { .. }
                | Self::Receive { .. }
                | Self::ReceiveReconnect { .. }
        )
    }

    fn cancelling(&self) -> bool {
        matches!(
            self,
            Self::CancelHandshake
                | Self::CancelHandshakeReconnect
                | Self::CancelReceive
                | Self::CancelReceiveReconnect
        )
    }

    fn cancelling_effect(&self, effect: &Self::Effect) -> bool {
        (matches!(effect, SubscribeEffect::Handshake { .. })
            && matches!(self, Self::CancelHandshake { .. }))
            || (matches!(effect, SubscribeEffect::HandshakeReconnect { .. })
                && matches!(self, Self::CancelHandshakeReconnect { .. }))
            || (matches!(effect, SubscribeEffect::Receive { .. })
                && matches!(self, Self::CancelReceive { .. }))
            || (matches!(effect, SubscribeEffect::ReceiveReconnect { .. })
                && matches!(self, Self::CancelReceiveReconnect { .. }))
    }
}

impl Display for SubscribeEffectInvocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Handshake { .. } => write!(f, "HANDSHAKE"),
            Self::CancelHandshake => write!(f, "CANCEL_HANDSHAKE"),
            Self::HandshakeReconnect { .. } => write!(f, "HANDSHAKE_RECONNECT"),
            Self::CancelHandshakeReconnect => write!(f, "CANCEL_HANDSHAKE_RECONNECT"),
            Self::Receive { .. } => write!(f, "RECEIVE_MESSAGES"),
            Self::CancelReceive { .. } => write!(f, "CANCEL_RECEIVE_MESSAGES"),
            Self::ReceiveReconnect { .. } => write!(f, "RECEIVE_RECONNECT"),
            Self::CancelReceiveReconnect { .. } => write!(f, "CANCEL_RECEIVE_RECONNECT"),
            Self::EmitStatus(status) => write!(f, "EMIT_STATUS({})", status),
            Self::EmitMessages(messages) => write!(f, "EMIT_MESSAGES({:?})", messages),
        }
    }
}
