use crate::core::Runtime as Trait;
use std::future::Future;

/// Spawns tasks on global tokio executor.
#[derive(Debug, Clone)]
pub struct Runtime;

impl Trait for Runtime {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self
    }
}
