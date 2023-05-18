use crate::{
    core::{event_engine::EffectHandler, PubNubError},
    dx::subscribe::{
        event_engine::{SubscribeEffect, SubscribeEffectInvocation},
        SubscribeCursor,
    },
};

type HandshakeFunction = fn(
    channels: Option<Vec<String>>,
    channel_groups: Option<Vec<String>>,
    attempt: u8,
    reason: Option<PubNubError>,
);
type ReceiveFunction = fn(
    channels: Option<Vec<String>>,
    channel_groups: Option<Vec<String>>,
    cursor: SubscribeCursor,
    attempt: u8,
    reason: Option<PubNubError>,
);

pub struct SubscribeEffectHandler {
    /// Handshake function pointer.
    handshake: HandshakeFunction,

    /// Receive updates function pointer.
    receive: ReceiveFunction,
}

impl SubscribeEffectHandler {
    pub fn new(handshake: HandshakeFunction, receive: ReceiveFunction) -> Self {
        SubscribeEffectHandler { handshake, receive }
    }
}

impl EffectHandler<SubscribeEffectInvocation, SubscribeEffect> for SubscribeEffectHandler {
    fn create(&self, invocation: &SubscribeEffectInvocation) -> Option<SubscribeEffect> {
        match invocation {
            SubscribeEffectInvocation::Handshake { .. } => todo!("Provide handshake effect"),
            SubscribeEffectInvocation::HandshakeReconnect { .. } => {
                todo!("Provide handshake reconnect effect")
            }
            SubscribeEffectInvocation::Receive { .. } => todo!("Provide receive effect"),
            SubscribeEffectInvocation::ReceiveReconnect { .. } => {
                todo!("Provide receive reconnect effect")
            }
            SubscribeEffectInvocation::EmitStatus(_) => todo!("Provide emit status effect"),
            SubscribeEffectInvocation::EmitMessages(_) => todo!("Provide emit messages effect"),
            _ => None,
        }
    }
}
