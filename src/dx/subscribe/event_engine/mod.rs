//! Subscribe Event Engine module

use crate::{
    core::event_engine::EventEngine,
    lib::alloc::{string::String, vec::Vec},
};

#[doc(inline)]
pub(crate) use effects::SubscribeEffect;
pub(crate) mod effects;

#[doc(inline)]
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

#[doc(inline)]
#[allow(unused_imports)]
pub(in crate::dx::subscribe) use types::{SubscriptionInput, SubscriptionParams};
pub(in crate::dx::subscribe) mod types;

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
            SubscribeState::Handshaking { input, .. }
            | SubscribeState::HandshakeReconnecting { input, .. }
            | SubscribeState::HandshakeStopped { input, .. }
            | SubscribeState::HandshakeFailed { input, .. }
            | SubscribeState::Receiving { input, .. }
            | SubscribeState::ReceiveReconnecting { input, .. }
            | SubscribeState::ReceiveStopped { input, .. }
            | SubscribeState::ReceiveFailed { input, .. } => {
                (input.channels(), input.channel_groups())
            }
            _ => (None, None),
        }
    }
}
