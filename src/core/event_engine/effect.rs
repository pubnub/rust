use crate::{
    core::event_engine::EffectInvocation,
    lib::alloc::{boxed::Box, string::String, vec::Vec},
};

#[async_trait::async_trait]
pub(crate) trait Effect: Send + Sync {
    type Invocation: EffectInvocation;

    /// Unique effect identifier.
    fn id(&self) -> String;

    /// Run actual effect implementation.
    async fn run(&self) -> Vec<<Self::Invocation as EffectInvocation>::Event>;

    /// Cancel any ongoing effect's work.
    fn cancel(&self);
}
