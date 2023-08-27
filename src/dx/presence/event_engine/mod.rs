//! # Presence Event Engine module

use crate::core::event_engine::EventEngine;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use effects::PresenceEffect;
pub(crate) mod effects;

#[doc(inline)]
pub(crate) use effect_handler::PresenceEffectHandler;
pub(crate) mod effect_handler;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use invocation::PresenceEffectInvocation;
pub(crate) mod invocation;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use event::PresenceEvent;
pub(crate) mod event;

#[doc(inline)]
#[allow(unused_imports)]
pub(crate) use state::PresenceState;
pub(crate) mod state;

#[doc(inline)]
#[allow(unused_imports)]
pub(in crate::dx::presence) use types::{PresenceInput, PresenceParameters};
pub(in crate::dx::presence) mod types;

pub(crate) type PresenceEventEngine =
    EventEngine<PresenceState, PresenceEffectHandler, PresenceEffect, PresenceEffectInvocation>;
