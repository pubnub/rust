//! # Presence event engine effects handler.
//!
//! The module contains the [`PresenceEffectHandler`] type, which is used by
//! event engine for

use async_channel::Sender;
use spin::RwLock;
use uuid::Uuid;

use crate::{
    core::{event_engine::EffectHandler, RequestRetryConfiguration},
    lib::{
        alloc::sync::Arc,
        core::fmt::{Debug, Formatter, Result},
    },
    presence::event_engine::{
        effects::{HeartbeatEffectExecutor, LeaveEffectExecutor, WaitEffectExecutor},
        PresenceEffect, PresenceEffectInvocation,
    },
};

/// Presence effect handler.
///
/// Handler responsible for effects implementation and creation in response on
/// effect invocation.
pub(crate) struct PresenceEffectHandler {
    /// Heartbeat call function pointer.
    heartbeat_call: Arc<HeartbeatEffectExecutor>,

    /// Delayed heartbeat call function pointer.
    delayed_heartbeat_call: Arc<HeartbeatEffectExecutor>,

    /// Leave function pointer.
    leave_call: Arc<LeaveEffectExecutor>,

    /// Heartbeat interval wait function pointer.
    wait_call: Arc<WaitEffectExecutor>,

    /// Retry policy.
    retry_policy: RequestRetryConfiguration,

    /// Cancellation channel.
    cancellation_channel: Sender<String>,
}

impl PresenceEffectHandler {
    /// Create presence effect handler.
    pub fn new(
        heartbeat_call: Arc<HeartbeatEffectExecutor>,
        delayed_heartbeat_call: Arc<HeartbeatEffectExecutor>,
        leave_call: Arc<LeaveEffectExecutor>,
        wait_call: Arc<WaitEffectExecutor>,
        retry_policy: RequestRetryConfiguration,
        cancellation_channel: Sender<String>,
    ) -> Self {
        Self {
            heartbeat_call,
            delayed_heartbeat_call,
            leave_call,
            wait_call,
            retry_policy,
            cancellation_channel,
        }
    }
}

impl EffectHandler<PresenceEffectInvocation, PresenceEffect> for PresenceEffectHandler {
    fn create(&self, invocation: &PresenceEffectInvocation) -> Option<PresenceEffect> {
        match invocation {
            PresenceEffectInvocation::Heartbeat { input } => Some(PresenceEffect::Heartbeat {
                id: Uuid::new_v4().to_string(),
                input: input.clone(),
                executor: self.heartbeat_call.clone(),
            }),
            PresenceEffectInvocation::DelayedHeartbeat {
                input,
                attempts,
                reason,
            } => Some(PresenceEffect::DelayedHeartbeat {
                id: Uuid::new_v4().to_string(),
                cancelled: RwLock::new(false),
                input: input.clone(),
                attempts: *attempts,
                reason: reason.clone(),
                retry_policy: self.retry_policy.clone(),
                executor: self.delayed_heartbeat_call.clone(),
                cancellation_channel: self.cancellation_channel.clone(),
            }),
            PresenceEffectInvocation::Leave { input } => Some(PresenceEffect::Leave {
                id: Uuid::new_v4().to_string(),
                input: input.clone(),
                executor: self.leave_call.clone(),
            }),
            PresenceEffectInvocation::Wait { input } => Some(PresenceEffect::Wait {
                id: Uuid::new_v4().to_string(),
                cancelled: RwLock::new(false),
                input: input.clone(),
                executor: self.wait_call.clone(),
                cancellation_channel: self.cancellation_channel.clone(),
            }),
            _ => None,
        }
    }
}

impl Debug for PresenceEffectHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "PresenceEffectHandler {{}}")
    }
}
