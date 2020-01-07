use crate::core::Runtime;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

/// Spawns tasks on the specified tokio runtime.
#[derive(Debug, Clone)]
pub struct Tokio {
    runtime: Arc<TokioRuntime>,
}

impl From<TokioRuntime> for Tokio {
    #[must_use]
    fn from(rt: TokioRuntime) -> Self {
        Self {
            runtime: Arc::new(rt),
        }
    }
}

impl Runtime for Tokio {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
    }
}

impl Default for Tokio {
    #[must_use]
    fn default() -> Self {
        let runtime = TokioRuntime::new().expect("unable to initialize tokio runtime");
        Self::from(runtime)
    }
}
