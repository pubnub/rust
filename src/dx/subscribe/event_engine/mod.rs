//! Subscribe Event Engine module

#[doc(inline)]
pub(crate) use effects::SubscribeEffect;
pub(crate) mod effects;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use effect_handler::SubscribeEffectHandler;
pub(crate) mod effect_handler;

#[doc(inline)]
pub(crate) use invocation::SubscribeEffectInvocation;
pub(crate) mod invocation;

#[doc(inline)]
pub(crate) use event::SubscribeEvent;
pub(crate) mod event;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use state::SubscribeState;
pub(crate) mod state;

use crate::{
    core::event_engine::EventEngine,
    lib::alloc::{string::String, vec::Vec},
};

pub(crate) type SubscribeEventEngine =
    EventEngine<SubscribeState, SubscribeEffectHandler, SubscribeEffect, SubscribeEffectInvocation>;

impl
    EventEngine<SubscribeState, SubscribeEffectHandler, SubscribeEffect, SubscribeEffectInvocation>
{
    #[allow(dead_code)]
    pub(in crate::dx::subscribe) fn current_subscription(
        &self,
    ) -> (Option<Vec<String>>, Option<Vec<String>>) {
        match self.current_state() {
            SubscribeState::Handshaking {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::HandshakeReconnecting {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::HandshakeStopped {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::HandshakeFailed {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::Receiving {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::ReceiveReconnecting {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::ReceiveStopped {
                channels,
                channel_groups,
                ..
            }
            | SubscribeState::ReceiveFailed {
                channels,
                channel_groups,
                ..
            } => (channels, channel_groups),
            _ => (None, None),
        }
    }
}
