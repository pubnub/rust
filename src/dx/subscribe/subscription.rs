use crate::{
    core::{blocking::Transport, event_engine::EventEngine},
    PubNubGenericClient,
};

use super::event_engine::{SubscribeEffectHandler, SubscribeEvent, SubscribeState};

pub(crate) struct Subscription {
    current_state: SubscribeState,
    handler: SubscribeEffectHandler,
}

impl Subscription {
    pub(crate) fn new<T>(client: PubNubGenericClient<T>) -> Self
    where
        T: Transport,
    {
        let handshake =
            |channels: &_, channel_groups: &_, attempt, reason| SubscribeEvent::HandshakeSuccess {
                cursor: super::SubscribeCursor {
                    timetoken: 0,
                    region: 0,
                },
            };

        // TODO: implement
        let receive = |_, _, _, _, _| {};

        Self {
            current_state: SubscribeState::Unsubscribed,
            handler: SubscribeEffectHandler::new(handshake, receive),
        }
    }
}
