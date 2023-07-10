use futures::future::BoxFuture;

use crate::core::PubNubError;

use super::Event;

pub(crate) enum EffectExecution<E>
where
    E: Event,
{
    /// No effect execution.
    None,

    /// Async effect execution.
    Async(BoxFuture<'static, Result<Vec<E>, PubNubError>>),

    /// Sync effect execution.
    Sync(Box<dyn Fn() -> Result<Vec<E>, PubNubError>>), // TODO: may it work?
}
