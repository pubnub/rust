//! # Presence event engine effect module.

use async_channel::Sender;
use futures::future::BoxFuture;
use spin::RwLock;

use crate::{
    core::{
        event_engine::{Effect, EffectInvocation},
        PubNubError,
    },
    lib::{
        alloc::{string::String, sync::Arc, vec::Vec},
        core::fmt::{Debug, Formatter},
    },
    presence::{
        event_engine::{PresenceEffectInvocation, PresenceInput, PresenceParameters},
        HeartbeatResult, LeaveResult,
    },
};

mod heartbeat;
mod leave;
mod wait;

/// Heartbeat effect executor.
///
/// The provided closure should pass [`PresenceParameters`] to the heartbeat
/// [`PubNub API`] endpoint and return processed results.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
pub(in crate::dx::presence) type HeartbeatEffectExecutor = dyn Fn(PresenceParameters) -> BoxFuture<'static, Result<HeartbeatResult, PubNubError>>
    + Send
    + Sync;

/// Wait effect executor.
///
/// The provided closure should provide the ability to wait a specified amount
/// of time before further program execution.
pub(in crate::dx::presence) type WaitEffectExecutor =
    dyn Fn(&str) -> BoxFuture<'static, Result<(), PubNubError>> + Send + Sync;

/// Leave effect executor.
///
/// The provided closure should pass [`PresenceParameters`] to the presence
/// leave [`PubNub API`] endpoint and return processed results.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
pub(in crate::dx::presence) type LeaveEffectExecutor = dyn Fn(PresenceParameters) -> BoxFuture<'static, Result<LeaveResult, PubNubError>>
    + Send
    + Sync;

/// Presence state machine effects.
pub(crate) enum PresenceEffect {
    /// Heartbeat effect invocation.
    Heartbeat {
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,

        /// Executor function.
        ///
        /// Function which will be used to execute heartbeat.
        executor: Arc<HeartbeatEffectExecutor>,
    },

    /// Leave effect invocation.
    Leave {
        /// Unique effect identifier.
        id: String,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// should leave.
        input: PresenceInput,

        /// Executor function.
        ///
        /// Function which will be used to execute leave.
        executor: Arc<LeaveEffectExecutor>,
    },

    /// Delay effect invocation.
    Wait {
        /// Unique effect identifier.
        id: String,

        /// Whether wait effect has been cancelled or not.
        cancelled: RwLock<bool>,

        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced after delay.
        input: PresenceInput,

        /// Cancellation channel.
        ///
        /// Channel which will be used to cancel effect execution.
        cancellation_channel: Sender<String>,

        /// Executor function.
        ///
        /// Function which will be used to execute wait.
        executor: Arc<WaitEffectExecutor>,
    },
}

impl Debug for PresenceEffect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Heartbeat { input, .. } => write!(
                f,
                "PresenceEffect::Heartbeat {{ channels: {:?}, channel groups: {:?}}}",
                input.channels(),
                input.channel_groups()
            ),
            Self::Leave { input, .. } => write!(
                f,
                "PresenceEffect::Leave {{ channels: {:?}, channel groups: {:?}}}",
                input.channels(),
                input.channel_groups()
            ),
            Self::Wait { input, .. } => write!(
                f,
                "PresenceEffect::Wait {{ channels: {:?}, channel groups: {:?}}}",
                input.channels(),
                input.channel_groups()
            ),
        }
    }
}

#[async_trait::async_trait]
impl Effect for PresenceEffect {
    type Invocation = PresenceEffectInvocation;

    fn name(&self) -> String {
        match self {
            Self::Heartbeat { .. } => "HEARTBEAT",
            Self::Leave { .. } => "LEAVE",
            Self::Wait { .. } => "WAIT",
        }
        .into()
    }

    fn id(&self) -> String {
        match self {
            Self::Heartbeat { id, .. } | Self::Leave { id, .. } | Self::Wait { id, .. } => id,
        }
        .into()
    }

    async fn run(&self) -> Vec<<Self::Invocation as EffectInvocation>::Event> {
        match self {
            Self::Heartbeat {
                input, executor, ..
            } => heartbeat::execute(input, executor).await,
            Self::Leave {
                input, executor, ..
            } => leave::execute(input, executor).await,
            Self::Wait { id, executor, .. } => wait::execute(id, executor).await,
        }
    }

    fn cancel(&self) {
        match self {
            PresenceEffect::Wait {
                id,
                cancelled,
                cancellation_channel,
                ..
            } => {
                {
                    let mut cancelled_slot = cancelled.write();
                    *cancelled_slot = true;
                }
                cancellation_channel
                    .send_blocking(id.clone())
                    .expect("Cancellation pipe is broken!");
            }
            _ => { /* cannot cancel other effects */ }
        }
    }

    fn is_cancelled(&self) -> bool {
        match self {
            Self::Wait { cancelled, .. } => *cancelled.read(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn send_wait_cancellation_wait_notification() {
        let (tx, rx) = async_channel::bounded(1);

        let effect = PresenceEffect::Wait {
            id: Uuid::new_v4().to_string(),
            cancelled: RwLock::new(false),
            input: PresenceInput::new(&None, &None),
            executor: Arc::new(|_| Box::pin(async move { Ok(()) })),
            cancellation_channel: tx,
        };

        effect.cancel();
        assert_eq!(rx.recv().await.unwrap(), effect.id())
    }
}
