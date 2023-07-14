//! # Futures implementation using Tokio runtime
//!
//! This module contains [`TokioSpawner`] type.
//!
//! It requires the [`future_tokio` feature] to be enabled.
//!
//! [`future_tokio` feature]: ../index.html#features

use crate::core::runtime::Runtime;

/// Tokio-based `async` tasks spawner.
#[derive(Clone, Debug)]
pub struct TokioRuntime;

impl Runtime for TokioRuntime {
    fn spawn<R>(&self, future: impl futures::Future<Output = R> + Send + 'static)
    where
        R: Send + 'static,
    {
        tokio::spawn(future);
    }
}
