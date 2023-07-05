use crate::core::event_engine::{Effect, EffectInvocation};

pub(crate) trait EffectHandler<I, EF>
where
    I: EffectInvocation + Send + Sync,
    EF: Effect + Send + Sync,
{
    /// Create effect using information of effect `invocation`.
    fn create(&self, invocation: &I) -> Option<EF>;
}
