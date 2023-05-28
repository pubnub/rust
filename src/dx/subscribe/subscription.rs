use crate::{
    core::blocking::Transport,
    dx::subscribe::event_engine::{
        effect_handler::{HandshakeFunction, ReceiveFunction, SubscribeEffectHandler},
        SubscribeEvent, SubscribeState,
    },
    lib::alloc::vec,
    PubNubGenericClient,
};

pub(crate) struct Subscription {
    current_state: SubscribeState,
    handler: SubscribeEffectHandler,
}

impl Subscription {
    pub(crate) fn new<T>(client: PubNubGenericClient<T>) -> Self
    where
        T: Transport,
    {
        let handshake: HandshakeFunction = |channels: &_, channel_groups: &_, attempt, reason| {
            vec![SubscribeEvent::HandshakeSuccess {
                cursor: super::SubscribeCursor {
                    timetoken: 0,
                    region: 0,
                },
            }]
        };

        // TODO: implement
        let receive: ReceiveFunction = |&_, &_, &_, _, _| vec![];

        Self {
            current_state: SubscribeState::Unsubscribed,
            handler: SubscribeEffectHandler::new(handshake, receive),
        }
    }
}
