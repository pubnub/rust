use crate::{
    core::{event_engine::EffectInvocation, PubNubError},
    dx::subscribe::{event_engine::SubscribeEffect, SubscribeCursor, SubscribeStatus},
    lib::{
        alloc::{string::String, vec::Vec},
        core::fmt::{Formatter, Result},
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
        /// Optional list of channels.
        ///
        /// List of channels which will be source of real-time updates after
        /// initial subscription completion.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which will be source of real-time updates
        /// after initial subscription completion.
        channel_groups: Option<Vec<String>>,
    },

    /// Cancel initial subscribe effect invocation.
    CancelHandshake,

    /// Retry initial subscribe effect invocation.
    HandshakeReconnect {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed initial
        /// subscription.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// initial subscription.
        channel_groups: Option<Vec<String>>,

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
        /// Optional list of channels.
        ///
        /// List of channels for which real-time updates will be delivered.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which real-time updates will be
        /// delivered.
        channel_groups: Option<Vec<String>>,

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
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed receive
        /// updates.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// receive updates.
        channel_groups: Option<Vec<String>>,

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
    EmitMessages(Vec<String>),
}

impl EffectInvocation for SubscribeEffectInvocation {
    type Effect = SubscribeEffect;

    fn id(&self) -> &str {
        match self {
            Self::Handshake { .. } => "Handshake",
            Self::CancelHandshake => "CancelHandshake",
            Self::HandshakeReconnect { .. } => "HandshakeReconnect",
            Self::CancelHandshakeReconnect => "CancelHandshakeReconnect",
            Self::Receive { .. } => "Receive",
            Self::CancelReceive { .. } => "CancelReceive",
            Self::ReceiveReconnect { .. } => "ReceiveReconnect",
            Self::CancelReceiveReconnect { .. } => "CancelReceiveReconnect",
            Self::EmitStatus(_status) => "EmitStatus",
            Self::EmitMessages(_messages) => "EmitMessages",
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

impl core::fmt::Display for SubscribeEffectInvocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Handshake { .. } => write!(f, "Handshake"),
            Self::CancelHandshake => write!(f, "CancelHandshake"),
            Self::HandshakeReconnect { .. } => write!(f, "HandshakeReconnect"),
            Self::CancelHandshakeReconnect => write!(f, "CancelHandshakeReconnect"),
            Self::Receive { .. } => write!(f, "Receive"),
            Self::CancelReceive { .. } => write!(f, "CancelReceive"),
            Self::ReceiveReconnect { .. } => write!(f, "ReceiveReconnect"),
            Self::CancelReceiveReconnect { .. } => write!(f, "CancelReceiveReconnect"),
            Self::EmitStatus(status) => write!(f, "EmitStatus({})", status),
            Self::EmitMessages(messages) => write!(f, "EmitMessages({:?})", messages),
        }
    }
}
