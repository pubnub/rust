use crate::runtime::Runtime as Trait;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

#[derive(Debug, Clone)]
pub struct Runtime {
    runtime: Arc<TokioRuntime>,
}

impl Trait for Runtime {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
    }
}

impl Default for Runtime {
    fn default() -> Self {
        let runtime = TokioRuntime::new().expect("unable to initialize tokio runtime");
        Self {
            runtime: Arc::new(runtime),
        }
    }
}
