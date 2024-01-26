//! # Subscribe event engine effect module.

use async_channel::Sender;
use futures::future::BoxFuture;

use crate::{
    core::{event_engine::Effect, PubNubError, RequestRetryConfiguration},
    dx::subscribe::{
        event_engine::{
            types::{SubscriptionInput, SubscriptionParams},
            SubscribeEffectInvocation, SubscribeEvent,
        },
        result::{SubscribeResult, Update},
        ConnectionStatus, SubscriptionCursor,
    },
    lib::{
        alloc::{string::String, sync::Arc, vec::Vec},
        core::fmt::{Debug, Formatter},
    },
};

mod emit_messages;
mod emit_status;
mod handshake;
mod handshake_reconnection;
mod receive;
mod receive_reconnection;

/// `SubscribeEffectExecutor` is a trait alias representing a type that executes
/// subscribe effects.
///
/// It takes a `SubscriptionParams` as input and returns a `BoxFuture` that
/// resolves to a `Result` of `SubscribeResult` or `PubNubError`.
///
/// This trait alias is `Send` and `Sync`, allowing it to be used across
/// multiple threads safely.
pub(in crate::dx::subscribe) type SubscribeEffectExecutor = dyn Fn(SubscriptionParams) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>
    + Send
    + Sync;

/// `EmitStatusEffectExecutor` is a trait alias representing a type that
/// executes emit status effects.
///
/// It takes a `SubscribeStatus` as input and does not return any value.
///
/// This trait alias is `Send` and `Sync`, allowing it to be used across
/// multiple threads safely.
pub(in crate::dx::subscribe) type EmitStatusEffectExecutor = dyn Fn(ConnectionStatus) + Send + Sync;

/// `EmitMessagesEffectExecutor` is a trait alias representing a type that
/// executes the effect of emitting messages.
///
/// It takes a vector of `Update` objects as input and does not return any
/// value.
///
/// This trait alias is `Send` and `Sync`, allowing it to be used across
/// multiple threads safely.
pub(in crate::dx::subscribe) type EmitMessagesEffectExecutor =
    dyn Fn(Vec<Update>, SubscriptionCursor) + Send + Sync;

// TODO: maybe move executor and cancellation_channel to super struct?
pub(crate) enum SubscribeEffect {
    /// Initial subscribe effect invocation.
    Handshake {
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and channel groups which will be
        /// source of real-time updates after initial subscription completion.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: Option<SubscriptionCursor>,

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
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and channel groups which has been
        /// used during recently failed initial subscription.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: Option<SubscriptionCursor>,

        /// Current initial subscribe retry attempt.
        ///
        /// Used to track overall number of initial subscription retry attempts.
        attempts: u8,

        /// Initial subscribe attempt failure reason.
        reason: PubNubError,

        /// Retry policy.
        retry_policy: RequestRetryConfiguration,

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
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and channel groups for which
        /// real-time updates will be delivered.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscriptionCursor,

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
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and channel groups which has been
        /// used during recently failed receive updates.
        input: SubscriptionInput,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscriptionCursor,

        /// Current receive retry attempt.
        ///
        /// Used to track overall number of receive updates retry attempts.
        attempts: u8,

        /// Receive updates attempt failure reason.
        reason: PubNubError,

        /// Retry policy.
        retry_policy: RequestRetryConfiguration,

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
        /// Unique effect identifier.
        id: String,

        /// Status which should be emitted.
        status: ConnectionStatus,

        /// Executor function.
        ///
        /// Function which will be used to execute receive updates.
        executor: Arc<EmitStatusEffectExecutor>,
    },

    /// Received updates notification effect invocation.
    EmitMessages {
        /// Unique effect identifier.
        id: String,

        /// Next time cursor.
        ///
        /// Cursor which should be used for next subscription loop.
        next_cursor: SubscriptionCursor,

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
            Self::Handshake { input, .. } => write!(
                f,
                "SubscribeEffect::Handshake {{ channels: {:?}, channel groups: {:?} }}",
                input.channels(),
                input.channel_groups()
            ),
            Self::HandshakeReconnect {
                input,
                attempts,
                reason,
                ..
            } => write!(
                f,
                "SubscribeEffect::HandshakeReconnect {{ channels: {:?}, channel groups: {:?}, \
                attempts: {attempts:?}, reason: {reason:?} }}",
                input.channels(),
                input.channel_groups()
            ),
            Self::Receive { input, cursor, .. } => write!(
                f,
                "SubscribeEffect::Receive {{ channels: {:?}, channel groups: {:?}, cursor: \
                {cursor:?} }}",
                input.channels(),
                input.channel_groups()
            ),
            Self::ReceiveReconnect {
                input,
                attempts,
                reason,
                ..
            } => write!(
                f,
                "SubscribeEffect::ReceiveReconnect {{ channels: {:?}, channel groups: {:?}, \
                attempts: {attempts:?}, reason: {reason:?} }}",
                input.channels(),
                input.channel_groups()
            ),
            Self::EmitStatus { status, .. } => {
                write!(f, "SubscribeEffect::EmitStatus {{ status: {status:?} }}")
            }
            Self::EmitMessages { updates, .. } => {
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

    fn name(&self) -> String {
        match self {
            Self::Handshake { .. } => "HANDSHAKE",
            Self::HandshakeReconnect { .. } => "HANDSHAKE_RECONNECT",
            Self::Receive { .. } => "RECEIVE_MESSAGES",
            Self::ReceiveReconnect { .. } => "RECEIVE_RECONNECT",
            Self::EmitStatus { .. } => "EMIT_STATUS",
            Self::EmitMessages { .. } => "EMIT_MESSAGES",
        }
        .into()
    }

    fn id(&self) -> String {
        match self {
            Self::Handshake { id, .. }
            | Self::HandshakeReconnect { id, .. }
            | Self::Receive { id, .. }
            | Self::ReceiveReconnect { id, .. }
            | Self::EmitStatus { id, .. }
            | Self::EmitMessages { id, .. } => id,
        }
        .into()
    }

    async fn run(&self) -> Vec<SubscribeEvent> {
        match self {
            Self::Handshake {
                id,
                input,
                cursor,
                executor,
                ..
            } => handshake::execute(input, cursor, id, executor).await,
            Self::HandshakeReconnect {
                id,
                input,
                cursor,
                attempts,
                reason,
                retry_policy,
                executor,
                ..
            } => {
                handshake_reconnection::execute(
                    input,
                    cursor,
                    *attempts,
                    reason.clone(), /* TODO: Does run function need to borrow self? Or we can
                                     * consume it? */
                    id,
                    retry_policy,
                    executor,
                )
                .await
            }
            Self::Receive {
                id,
                input,
                cursor,
                executor,
                ..
            } => receive::execute(input, cursor, id, executor).await,
            Self::ReceiveReconnect {
                id,
                input,
                cursor,
                attempts,
                reason,
                retry_policy,
                executor,
                ..
            } => {
                receive_reconnection::execute(
                    input,
                    cursor,
                    *attempts,
                    reason.clone(), /* TODO: Does run function need to borrow self? Or we can
                                     * consume it? */
                    id,
                    retry_policy,
                    executor,
                )
                .await
            }
            Self::EmitStatus {
                status, executor, ..
            } => emit_status::execute(status.clone(), executor).await,
            Self::EmitMessages {
                updates,
                executor,
                next_cursor,
                ..
            } => emit_messages::execute(next_cursor.clone(), updates.clone(), executor).await,
        }
    }

    fn cancel(&self) {
        match self {
            Self::Handshake {
                id,
                cancellation_channel,
                ..
            }
            | Self::HandshakeReconnect {
                id,
                cancellation_channel,
                ..
            }
            | Self::Receive {
                id,
                cancellation_channel,
                ..
            }
            | Self::ReceiveReconnect {
                id,
                cancellation_channel,
                ..
            } => {
                cancellation_channel
                    .send_blocking(id.clone())
                    .expect("cancellation pipe is broken!");
            }
            _ => { /* cannot cancel other effects */ }
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use futures::FutureExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn send_cancellation_notification() {
        let (tx, rx) = async_channel::bounded::<String>(1);

        let effect = SubscribeEffect::Handshake {
            id: Uuid::new_v4().to_string(),
            input: SubscriptionInput::new(&None, &None),
            cursor: None,
            executor: Arc::new(|_| {
                async move {
                    Ok(SubscribeResult {
                        cursor: SubscriptionCursor::default(),
                        messages: vec![],
                    })
                }
                .boxed()
            }),
            cancellation_channel: tx,
        };

        effect.cancel();

        assert_eq!(rx.recv().await.unwrap(), effect.id());
    }
}
