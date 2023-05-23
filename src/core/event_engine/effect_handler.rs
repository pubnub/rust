use crate::core::event_engine::{Effect, EffectInvocation};

pub(crate) trait EffectHandler<I, EF>
where
    I: EffectInvocation,
    EF: Effect,
{
    /// Create effect using information of effect `invocation`.
    fn create(&self, invocation: &I) -> Option<EF>;
}
