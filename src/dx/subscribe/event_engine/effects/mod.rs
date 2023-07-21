use crate::{
    core::{event_engine::Effect, PubNubError, RequestRetryPolicy},
    dx::subscribe::{
        event_engine::{SubscribeEffectInvocation, SubscribeEvent},
        result::{SubscribeResult, Update},
        SubscribeCursor, SubscribeStatus, SubscriptionParams,
    },
    lib::{
        alloc::{boxed::Box, string::String, sync::Arc, vec::Vec},
        core::fmt::{Debug, Formatter},
    },
};
use async_channel::Sender;
use futures::future::BoxFuture;

mod emit_messagess;
mod emit_status;
mod handshake;
mod handshake_reconnection;
mod receive;
mod receive_reconnection;

pub(in crate::dx::subscribe) type SubscribeEffectExecutor = dyn Fn(SubscriptionParams) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>
    + Send
    + Sync;

pub(in crate::dx::subscribe) type EmitStatusEffectExecutor = dyn Fn(SubscribeStatus) + Send + Sync;
pub(in crate::dx::subscribe) type EmitMessagesEffectExecutor = dyn Fn(Vec<Update>) + Send + Sync;

// TODO: maybe move executor and cancellation_channel to super struct?
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
        executor: Arc<SubscribeEffectExecutor>,

        /// Cancellation channel.
        ///
        /// Channel which will be used to cancel effect execution.
        cancellation_channel: Sender<String>,
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

        /// Retry policy.
        retry_policy: RequestRetryPolicy,

        /// Executor function.
        ///
        /// Function which will be used to execute initial subscription.
        executor: Arc<SubscribeEffectExecutor>,

        /// Cancellation channel.
        ///
        /// Channel which will be used to cancel effect execution.
        cancellation_channel: Sender<String>,
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
        executor: Arc<SubscribeEffectExecutor>,

        /// Cancellation channel.
        ///
        /// Channel which will be used to cancel effect execution.
        cancellation_channel: Sender<String>,
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

        /// Retry policy.
        retry_policy: RequestRetryPolicy,

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: Arc<SubscribeEffectExecutor>,

        /// Cancellation channel.
        ///
        /// Channel which will be used to cancel effect execution.
        cancellation_channel: Sender<String>,
    },

    /// Status change notification effect invocation.
    EmitStatus {
        /// Status which should be emitted.
        status: SubscribeStatus,

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: Arc<EmitStatusEffectExecutor>,
    },

    /// Received updates notification effect invocation.
    EmitMessages {
        /// Updates which should be emitted.
        updates: Vec<Update>,

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: Arc<EmitMessagesEffectExecutor>,
    },
}

impl Debug for SubscribeEffect {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SubscribeEffect::Handshake {
                channels,
                channel_groups,
                ..
            } => write!(
                f,
                "SubscribeEffect::Handshake {{ channels: {channels:?}, channel groups: \
                {channel_groups:?} }}"
            ),
            SubscribeEffect::HandshakeReconnect {
                channels,
                channel_groups,
                attempts,
                reason,
                ..
            } => write!(
                f,
                "SubscribeEffect::HandshakeReconnect {{ channels: {channels:?}, channel groups: \
                {channel_groups:?}, attempts: {attempts:?}, reason: {reason:?} }}"
            ),
            SubscribeEffect::Receive {
                channels,
                channel_groups,
                cursor,
                ..
            } => write!(
                f,
                "SubscribeEffect::Receive {{ channels: {channels:?}, channel groups: \
                {channel_groups:?}, cursor: {cursor:?} }}"
            ),
            SubscribeEffect::ReceiveReconnect {
                channels,
                channel_groups,
                attempts,
                reason,
                ..
            } => write!(
                f,
                "SubscribeEffect::ReceiveReconnect {{ channels: {channels:?}, channel groups: \
                {channel_groups:?}, attempts: {attempts:?}, reason: {reason:?} }}"
            ),
            SubscribeEffect::EmitStatus { status, .. } => {
                write!(f, "SubscribeEffect::EmitStatus {{ status: {status:?} }}")
            }
            SubscribeEffect::EmitMessages { updates, .. } => {
                write!(
                    f,
                    "SubscribeEffect::EmitMessages {{ messages: {updates:?} }}"
                )
            }
        }
    }
}

#[async_trait::async_trait]
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

    async fn run(&self) -> Vec<SubscribeEvent> {
        match self {
            SubscribeEffect::Handshake {
                channels,
                channel_groups,
                executor,
                ..
            } => handshake::execute(channels, channel_groups, &self.id(), executor).await,
            SubscribeEffect::HandshakeReconnect {
                channels,
                channel_groups,
                attempts,
                reason,
                retry_policy,
                executor,
                ..
            } => {
                handshake_reconnection::execute(
                    channels,
                    channel_groups,
                    *attempts,
                    reason.clone(), // TODO: Does run function need to borrow self? Or we can consume it?
                    &self.id(),
                    retry_policy,
                    executor,
                )
                .await
            }
            SubscribeEffect::Receive {
                channels,
                channel_groups,
                cursor,
                executor,
                ..
            } => receive::execute(channels, channel_groups, cursor, &self.id(), executor).await,
            SubscribeEffect::ReceiveReconnect {
                channels,
                channel_groups,
                cursor,
                attempts,
                reason,
                retry_policy,
                executor,
                ..
            } => {
                receive_reconnection::execute(
                    channels,
                    channel_groups,
                    cursor,
                    *attempts,
                    reason.clone(), // TODO: Does run function need to borrow self? Or we can consume it?
                    &self.id(),
                    retry_policy,
                    executor,
                )
                .await
            }
            SubscribeEffect::EmitStatus { status, executor } => {
                emit_status::execute(*status, executor).await
            }
            SubscribeEffect::EmitMessages { updates, executor } => {
                emit_messagess::execute(updates.clone(), executor).await
            }
        }
    }

    fn cancel(&self) {
        match self {
            SubscribeEffect::Handshake {
                cancellation_channel,
                ..
            }
            | SubscribeEffect::HandshakeReconnect {
                cancellation_channel,
                ..
            }
            | SubscribeEffect::Receive {
                cancellation_channel,
                ..
            }
            | SubscribeEffect::ReceiveReconnect {
                cancellation_channel,
                ..
            } => {
                cancellation_channel.send_blocking(self.id()).unwrap(); // TODO: result ;/
            }
            _ => { /* cannot cancel other effects */ }
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[tokio::test]
    async fn send_cancelation_notification() {
        let (tx, rx) = async_channel::bounded(1);

        let effect = SubscribeEffect::Handshake {
            channels: None,
            channel_groups: None,
            executor: Arc::new(|_| {
                Box::pin(async move {
                    Ok(SubscribeResult {
                        cursor: SubscribeCursor::default(),
                        messages: vec![],
                    })
                })
            }),
            cancellation_channel: tx,
        };

        effect.cancel();

        assert_eq!(rx.recv().await.unwrap(), effect.id())
    }
}
