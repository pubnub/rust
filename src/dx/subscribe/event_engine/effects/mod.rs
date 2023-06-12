use crate::dx::subscribe::event_engine::{SubscribeEffectInvocation, SubscribeEvent};
use crate::{
    core::{event_engine::Effect, PubNubError},
    dx::subscribe::{SubscribeCursor, SubscribeStatus},
    lib::alloc::{string::String, vec::Vec},
};

use super::effect_handler::EmitFunction;
use super::{HandshakeFunction, ReceiveFunction};

mod handshake;
mod handshake_reconnection;
mod receive;
mod receive_reconnection;

/// Subscription state machine effects.
#[allow(dead_code)]
pub(crate) enum SubscribeEffect {
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

        /// Executor function.
        ///
        /// Function which will be used to execute initial subscription.
        executor: HandshakeFunction,
    },

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

        /// Executor function.
        ///
        /// Function which will be used to execute initial subscription.
        executor: HandshakeFunction,
    },

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

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: ReceiveFunction,
    },

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

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: ReceiveFunction,
    },

    /// Status change notification effect invocation.
    EmitStatus {
        /// Current subscription status.
        ///
        /// Used to notify about subscription status changes.
        status: SubscribeStatus,

        /// Emiting function.
        ///
        /// Function which will be used to emit subscription status changes.
        executor: EmitFunction,
    },

    /// Received updates notification effect invocation.
    EmitMessages {
        /// Received Messages
        ///
        /// Messages ready to be emitted to the user.
        messages: Vec<String>,

        /// Emiting function.
        ///
        /// Function which will be used to emit subscription status changes.
        executor: EmitFunction,
    },
}

impl Effect for SubscribeEffect {
    type Invocation = SubscribeEffectInvocation;

    fn id(&self) -> String {
        // TODO: Identifiers need to be unique, so we won't cancel wrong effect
        match self {
            SubscribeEffect::Handshake { .. } => "HANDSHAKE_EFFECT".into(),
            SubscribeEffect::HandshakeReconnect { .. } => "HANDSHAKE_RECONNECT_EFFECT".into(),
            SubscribeEffect::Receive { .. } => "RECEIVE_EFFECT".into(),
            SubscribeEffect::ReceiveReconnect { .. } => "RECEIVE_RECONNECT_EFFECT".into(),
            SubscribeEffect::EmitStatus { .. } => "EMIT_STATUS_EFFECT".into(),
            SubscribeEffect::EmitMessages { .. } => "EMIT_MESSAGES_EFFECT".into(),
        }
    }
    fn run<F>(&self, mut f: F)
    where
        F: FnMut(Option<Vec<SubscribeEvent>>),
    {
        // TODO: Run actual effect implementation. Maybe Effect.run function need change something in arguments.
        let events = match self {
            SubscribeEffect::Handshake {
                channels,
                channel_groups,
                executor,
            } => handshake::execute(channels, channel_groups, *executor),
            SubscribeEffect::HandshakeReconnect {
                channels,
                channel_groups,
                attempts,
                reason,
                executor,
            } => handshake_reconnection::execute(
                channels,
                channel_groups,
                *attempts,
                reason.clone(), // TODO: Does run function need to borrow self? Or we can consume it?
                *executor,
            ),
            SubscribeEffect::Receive {
                channels,
                channel_groups,
                cursor,
                executor,
            } => receive::execute(channels, channel_groups, cursor, *executor),
            SubscribeEffect::ReceiveReconnect {
                channels,
                channel_groups,
                cursor,
                attempts,
                reason,
                executor,
            } => receive_reconnection::execute(
                channels,
                channel_groups,
                cursor,
                *attempts,
                reason.clone(), // TODO: Does run function need to borrow self? Or we can consume it?
                *executor,
            ),
            _ => {
                /* TODO: Implement other effects */
                None
            }
        };

        f(events);
    }

    fn cancel(&self) {
        // TODO: Cancellation required for corresponding SubscribeEffect variants.
    }
}
