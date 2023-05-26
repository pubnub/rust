use crate::{
    core::{event_engine::EffectHandler, PubNubError},
    dx::subscribe::{
        event_engine::{SubscribeEffect, SubscribeEffectInvocation},
        SubscribeCursor,
    },
    lib::alloc::{string::String, vec::Vec},
};

use super::SubscribeEvent;

pub(crate) type HandshakeFunction = fn(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: Option<PubNubError>,
) -> SubscribeEvent;

pub(crate) type ReceiveFunction = fn(
    channels: Option<Vec<String>>,
    channel_groups: Option<Vec<String>>,
    cursor: SubscribeCursor,
    attempt: u8,
    reason: Option<PubNubError>,
);

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
}

impl SubscribeEffectHandler {
    /// Create subscribe event handler.
    #[allow(dead_code)]
    pub fn new(handshake: HandshakeFunction, receive: ReceiveFunction) -> Self {
        SubscribeEffectHandler { handshake, receive }
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
            }),
            SubscribeEffectInvocation::Receive {
                channels,
                channel_groups,
                cursor,
            } => Some(SubscribeEffect::Receive {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: *cursor,
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
            }),
            SubscribeEffectInvocation::EmitStatus(status) => {
                // TODO: Provide emit status effect
                Some(SubscribeEffect::EmitStatus(*status))
            }
            SubscribeEffectInvocation::EmitMessages(messages) => {
                // TODO: Provide emit messages effect
                Some(SubscribeEffect::EmitMessages(messages.clone()))
            }
            _ => None,
        }
    }
}
