use crate::core::event_engine::{Effect, EffectInvocation};

pub trait EffectHandler<I, E>
where
    I: EffectInvocation,
    E: Effect,
{
    /// Create effect using information of effect `invocation`.
    fn create(&self, invocation: &I) -> Option<E>;
}
