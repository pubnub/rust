use crate::dx::subscribe::result::Update;
use crate::dx::subscribe::SubscribeStatus;
use crate::{
    core::{event_engine::EffectHandler, PubNubError},
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{
            event_engine::{SubscribeEffect, SubscribeEffectInvocation},
            result::SubscribeResult,
            SubscribeCursor,
        },
    },
    lib::alloc::{string::String, vec::Vec},
    PubNubGenericClient,
};
use futures::future::BoxFuture;

pub(crate) type HandshakeFunction<Transport> =
    fn(
        client: PubNubClientInstance<Transport>,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
        attempt: u8,
        reason: Option<PubNubError>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>;

pub(crate) type ReceiveFunction<Transport> =
    fn(
        client: PubNubClientInstance<Transport>,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
        cursor: &SubscribeCursor,
        attempt: u8,
        reason: Option<PubNubError>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>;

pub(crate) type EmitStatus<Transport> =
    fn(client: PubNubClientInstance<Transport>, status: &SubscribeStatus);

pub(crate) type EmitMessages<Transport> =
    fn(client: PubNubClientInstance<Transport>, messages: Vec<Update>);

/// Subscription effect handler.
///
/// Handler responsible for effects implementation and creation in response on
/// effect invocation.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SubscribeEffectHandler<Transport> {
    client: PubNubClientInstance<Transport>,

    /// Handshake function pointer.
    handshake: HandshakeFunction<Transport>,

    /// Receive updates function pointer.
    receive: ReceiveFunction<Transport>,

    /// Emit status function pointer.
    emit_status: EmitStatus<Transport>,

    /// Emit messages function pointer.
    emit_messages: EmitMessages<Transport>,
}

impl<Transport> SubscribeEffectHandler<Transport> {
    /// Create subscribe event handler.
    #[allow(dead_code)]
    pub fn new(
        client: PubNubClientInstance<Transport>,
        handshake: HandshakeFunction<Transport>,
        receive: ReceiveFunction<Transport>,
        emit_status: EmitStatus<Transport>,
        emit_messages: EmitMessages<Transport>,
    ) -> Self {
        SubscribeEffectHandler {
            client,
            handshake,
            receive,
            emit_status,
            emit_messages,
        }
    }
}

impl<Transport> EffectHandler<SubscribeEffectInvocation, SubscribeEffect>
    for SubscribeEffectHandler<Transport>
{
    fn create(&self, invocation: &SubscribeEffectInvocation) -> Option<SubscribeEffect> {
        match invocation {
            SubscribeEffectInvocation::Handshake {
                channels,
                channel_groups,
            } => Some(SubscribeEffect::Handshake {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                executor: Box::new(|channels, groups, retry, reason| {
                    (self.handshake)(self.client.clone(), channels, channel_groups, retry, reason)
                }),
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
                executor: Box::new(|channels, groups, retry, reason| {
                    let s = (self.handshake)(
                        self.client.clone(),
                        channels,
                        channel_groups,
                        retry,
                        reason,
                    );
                }),
            }),
            SubscribeEffectInvocation::Receive {
                channels,
                channel_groups,
                cursor,
            } => Some(SubscribeEffect::Receive {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: cursor.clone(),
                executor: Box::new(|channels, groups, cursor, retry, reason| {
                    (self.receive)(
                        self.client.clone(),
                        channels,
                        channel_groups,
                        cursor,
                        retry,
                        reason,
                    )
                }),
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
                cursor: cursor.clone(),
                attempts: *attempts,
                reason: reason.clone(),
                executor: Box::new(|channels, groups, cursor, retry, reason| {
                    (self.receive)(
                        self.client.clone(),
                        channels,
                        channel_groups,
                        cursor,
                        retry,
                        reason,
                    )
                }),
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
