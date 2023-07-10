use crate::{
    core::event_engine::EffectInvocation,
    lib::alloc::{string::String, vec::Vec},
};

use super::effect_execution::EffectExecution;

pub(crate) trait Effect: Send + Sync {
    type Invocation: EffectInvocation;

    /// Unique effect identifier.
    fn id(&self) -> String;

    fn run(&self) -> EffectExecution<<Self::Invocation as EffectInvocation>::Event>;

    /// Cancel any ongoing effect's work.
    fn cancel(&self);
}
