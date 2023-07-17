use crate::{
    core::event_engine::EffectInvocation,
    lib::alloc::{string::String, vec::Vec},
};

#[async_trait::async_trait]
pub(crate) trait Effect: Send + Sync {
    type Invocation: EffectInvocation;

    /// Unique effect identifier.
    fn id(&self) -> String;

    async fn run<F>(&self, f: F)
    where
        F: FnOnce(Vec<<Self::Invocation as EffectInvocation>::Event>) + Send + 'static;

    /// Cancel any ongoing effect's work.
    fn cancel(&self);
}
