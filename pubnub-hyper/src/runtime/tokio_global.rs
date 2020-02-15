//! Tokio global executor runtime.

use crate::core::Runtime;
use std::future::Future;

/// Spawns tasks on global tokio executor.
#[derive(Debug, Clone, Copy)]
pub struct TokioGlobal;

impl Runtime for TokioGlobal {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}

impl Default for TokioGlobal {
    #[must_use]
    fn default() -> Self {
        Self
    }
}
