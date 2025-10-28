use async_channel::Sender;
use spin::rwlock::RwLock;
use uuid::Uuid;

use crate::{
    core::event_engine::EffectHandler,
    dx::subscribe::event_engine::{
        effects::{EmitMessagesEffectExecutor, EmitStatusEffectExecutor, SubscribeEffectExecutor},
        SubscribeEffect, SubscribeEffectInvocation,
    },
    lib::{
        alloc::{string::String, sync::Arc},
        core::fmt::{Debug, Formatter, Result},
    },
};

/// Subscription effect handler.
///
/// Handler responsible for effects implementation and creation in response on
/// effect invocation.
pub(crate) struct SubscribeEffectHandler {
    /// Subscribe call function pointer.
    subscribe_call: Arc<SubscribeEffectExecutor>,

    /// Emit status function pointer.
    emit_status: Arc<EmitStatusEffectExecutor>,

    /// Emit messages function pointer.
    emit_messages: Arc<EmitMessagesEffectExecutor>,

    /// Cancellation channel.
    cancellation_channel: Sender<String>,
}

impl SubscribeEffectHandler {
    /// Create subscribe effect handler.
    pub fn new(
        subscribe_call: Arc<SubscribeEffectExecutor>,
        emit_status: Arc<EmitStatusEffectExecutor>,
        emit_messages: Arc<EmitMessagesEffectExecutor>,
        cancellation_channel: Sender<String>,
    ) -> Self {
        Self {
            subscribe_call,
            emit_status,
            emit_messages,
            cancellation_channel,
        }
    }
}

impl EffectHandler<SubscribeEffectInvocation, SubscribeEffect> for SubscribeEffectHandler {
    fn create(&self, invocation: &SubscribeEffectInvocation) -> Option<SubscribeEffect> {
        match invocation {
            SubscribeEffectInvocation::Handshake { input, cursor } => {
                Some(SubscribeEffect::Handshake {
                    id: Uuid::new_v4().to_string(),
                    cancelled: RwLock::new(false),
                    input: input.clone(),
                    cursor: cursor.clone(),
                    executor: self.subscribe_call.clone(),
                    cancellation_channel: self.cancellation_channel.clone(),
                })
            }
            SubscribeEffectInvocation::Receive { input, cursor } => {
                Some(SubscribeEffect::Receive {
                    id: Uuid::new_v4().to_string(),
                    cancelled: RwLock::new(false),
                    input: input.clone(),
                    cursor: cursor.clone(),
                    executor: self.subscribe_call.clone(),
                    cancellation_channel: self.cancellation_channel.clone(),
                })
            }
            SubscribeEffectInvocation::EmitStatus(status) => Some(SubscribeEffect::EmitStatus {
                id: Uuid::new_v4().to_string(),
                status: status.clone(),
                executor: self.emit_status.clone(),
            }),
            SubscribeEffectInvocation::EmitMessages(messages, cursor) => {
                Some(SubscribeEffect::EmitMessages {
                    id: Uuid::new_v4().to_string(),
                    next_cursor: cursor.clone(),
                    updates: messages.clone(),
                    executor: self.emit_messages.clone(),
                })
            }
            _ => None,
        }
    }
}

impl Debug for SubscribeEffectHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "SubscribeEffectHandler {{}}")
    }
}
