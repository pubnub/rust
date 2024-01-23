//! # Presence Event Engine module

use crate::core::event_engine::EventEngine;

#[doc(inline)]
pub(crate) use effects::PresenceEffect;
pub(crate) mod effects;

#[doc(inline)]
pub(crate) use effect_handler::PresenceEffectHandler;
pub(crate) mod effect_handler;

#[doc(inline)]
pub(crate) use invocation::PresenceEffectInvocation;
pub(crate) mod invocation;

#[doc(inline)]
pub(crate) use event::PresenceEvent;
pub(crate) mod event;

#[doc(inline)]
pub(crate) use state::PresenceState;
pub(crate) mod state;

#[doc(inline)]
pub(crate) use types::{PresenceInput, PresenceParameters};
mod types;

pub(crate) type PresenceEventEngine =
    EventEngine<PresenceState, PresenceEffectHandler, PresenceEffect, PresenceEffectInvocation>;
