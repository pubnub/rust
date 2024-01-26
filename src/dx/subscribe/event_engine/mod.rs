//! Subscribe Event Engine module

use crate::core::event_engine::EventEngine;

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
pub(crate) use state::SubscribeState;
pub(crate) mod state;

#[doc(inline)]
pub(in crate::dx::subscribe) use types::{SubscriptionInput, SubscriptionParams};
pub(in crate::dx::subscribe) mod types;

pub(crate) type SubscribeEventEngine =
    EventEngine<SubscribeState, SubscribeEffectHandler, SubscribeEffect, SubscribeEffectInvocation>;
