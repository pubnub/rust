use crate::{
    core::{event_engine::EffectHandler, PubNubError},
    dx::subscribe::{
        event_engine::{SubscribeEffect, SubscribeEffectInvocation},
        SubscribeCursor, SubscribeStatus,
    },
    lib::alloc::{string::String, vec::Vec},
};

use super::SubscribeEvent;

pub(crate) type HandshakeFunction = fn(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: Option<PubNubError>,
) -> Result<Vec<SubscribeEvent>, PubNubError>;

pub(crate) type ReceiveFunction = fn(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    attempt: u8,
    reason: Option<PubNubError>,
) -> Result<Vec<SubscribeEvent>, PubNubError>;

pub(crate) type EmitFunction = fn(data: EmitData) -> Result<(), PubNubError>;

/// Data emitted by subscription.
///
/// This data is emitted by subscription and is used to create subscription
/// events.
pub(crate) enum EmitData {
    /// Status emitted by subscription.
    SubscribeStatus(SubscribeStatus),

    /// Messages emitted by subscription.
    /// TODO: Replace String with Message type
    Messages(Vec<String>),
}

/// Subscription effect handler.
///
/// Handler responsible for effects implementation and creation in response on
/// effect invocation.
#[allow(dead_code)]
pub(crate) struct SubscribeEffectHandler {
    /// Handshake function pointer.
    handshake: HandshakeFunction,

    /// Receive updates function pointer.
    receive: ReceiveFunction,

    /// Emit data function pointer.
    emit: EmitFunction,
}

impl SubscribeEffectHandler {
    /// Create subscribe event handler.
    #[allow(dead_code)]
    pub fn new(handshake: HandshakeFunction, receive: ReceiveFunction, emit: EmitFunction) -> Self {
        SubscribeEffectHandler {
            handshake,
            receive,
            emit,
        }
    }
}

impl EffectHandler<SubscribeEffectInvocation, SubscribeEffect> for SubscribeEffectHandler {
    fn create(&self, invocation: &SubscribeEffectInvocation) -> Option<SubscribeEffect> {
        match invocation {
            SubscribeEffectInvocation::Handshake {
                channels,
                channel_groups,
            } => Some(SubscribeEffect::Handshake {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                executor: self.handshake,
            }),
            SubscribeEffectInvocation::HandshakeReconnect {
                channels,
                channel_groups,
                attempts,
                reason,
            } => Some(SubscribeEffect::HandshakeReconnect {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                attempts: *attempts,
                reason: reason.clone(),
                executor: self.handshake,
            }),
            SubscribeEffectInvocation::Receive {
                channels,
                channel_groups,
                cursor,
            } => Some(SubscribeEffect::Receive {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: *cursor,
                executor: self.receive,
            }),
            SubscribeEffectInvocation::ReceiveReconnect {
                channels,
                channel_groups,
                cursor,
                attempts,
                reason,
            } => Some(SubscribeEffect::ReceiveReconnect {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: *cursor,
                attempts: *attempts,
                reason: reason.clone(),
                executor: self.receive,
            }),
            SubscribeEffectInvocation::EmitStatus(status) => {
                // TODO: Provide emit status effect
                Some(SubscribeEffect::EmitStatus {
                    status: *status,
                    executor: self.emit,
                })
            }
            SubscribeEffectInvocation::EmitMessages(messages) => {
                // TODO: Provide emit messages effect
                Some(SubscribeEffect::EmitMessages {
                    messages: messages.clone(),
                    executor: self.emit,
                })
            }
            _ => None,
        }
    }
}
