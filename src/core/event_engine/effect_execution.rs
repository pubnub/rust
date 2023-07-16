use futures::future::BoxFuture;

use crate::{
    core::PubNubError,
    lib::alloc::{boxed::Box, vec, vec::Vec},
};

use super::Event;

#[allow(dead_code)]
pub(crate) enum EffectExecution<E>
where
    E: Event,
{
    /// No effect execution.
    None,

    /// Async effect execution.
    Async {
        future: BoxFuture<'static, Result<Vec<E>, PubNubError>>,
        then: Box<dyn FnOnce()>,
    },

    /// Sync effect execution.
    Sync(Box<dyn Fn() -> Result<Vec<E>, PubNubError>>), // TODO: may it work?
}

impl<E> EffectExecution<E>
where
    E: Event,
{
    #[allow(dead_code)]
    pub(crate) async fn execute_async(self) -> Result<Vec<E>, PubNubError> {
        match self {
            EffectExecution::None => Ok(vec![]),
            EffectExecution::Async { future, then } => {
                let result = future.await;

                then();

                result
            }
            EffectExecution::Sync(f) => f(),
        }
    }
}

#[cfg(test)]
mod should {
    use crate::lib::alloc::sync::Arc;
    use core::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    struct TestEvent;

    impl Event for TestEvent {
        fn id(&self) -> &str {
            "test_event"
        }
    }

    #[tokio::test]
    async fn call_then_statement_of_future() {
        let then_called = Arc::new(AtomicBool::new(false));
        let cloned = then_called.clone();

        let execution = EffectExecution::Async {
            future: Box::pin(async { Ok(vec![TestEvent]) }),
            then: Box::new(move || cloned.store(true, Ordering::Relaxed)),
        };

        execution.execute_async().await.unwrap();

        assert!(then_called.load(Ordering::Relaxed));
    }
}
