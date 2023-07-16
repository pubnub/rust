use crate::{
    core::event_engine::EffectInvocation,
    lib::alloc::{string::String, vec::Vec},
};

pub(crate) trait Effect: Send + Sync {
    type Invocation: EffectInvocation;

    /// Unique effect identifier.
    fn id(&self) -> String;

    fn run<F>(&self, f: F)
    where
        F: FnOnce(Vec<<Self::Invocation as EffectInvocation>::Event>) + 'static;

    /// Cancel any ongoing effect's work.
    fn cancel(&self);
}
