use crate::{
    core::{blocking::Transport, event_engine::EventEngine},
    dx::subscribe::event_engine::{
        effect_handler::{HandshakeFunction, ReceiveFunction, SubscribeEffectHandler},
        SubscribeState,
    },
    lib::alloc::vec,
    PubNubGenericClient,
};

use super::event_engine::{
    effect_handler::EmitFunction, SubscribeEffect, SubscribeEffectInvocation,
};

type SubscribeEngine =
    EventEngine<SubscribeState, SubscribeEffectHandler, SubscribeEffect, SubscribeEffectInvocation>;

/// Subscription that is responsible for getting messages from PubNub.
///
/// Subscription provides a way to get messages from PubNub. It is responsible
/// for handshake and receiving messages.
///
/// TODO: more description and examples
pub struct Subscription {
    engine: SubscribeEngine,
}

impl Subscription {
    pub(crate) fn subscribe<T>(_client: PubNubGenericClient<T>) -> Self
    where
        T: Transport,
    {
        // TODO: implementation is a part of the different task
        let handshake: HandshakeFunction = |_, _, _, _| Ok(vec![]);

        let receive: ReceiveFunction = |&_, &_, &_, _, _| Ok(vec![]);

        let emit: EmitFunction = |_| Ok(());

        Self {
            engine: SubscribeEngine::new(
                SubscribeEffectHandler::new(handshake, receive, emit),
                SubscribeState::Unsubscribed,
            ),
        }
    }
}
