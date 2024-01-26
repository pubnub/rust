//! # Futures implementation using Tokio runtime
//!
//! This module contains [`TokioSpawner`] type.
//!
//! It requires the [`future_tokio` feature] to be enabled.
//!
//! [`future_tokio` feature]: ../index.html#features

use crate::core::runtime::Runtime;

/// Tokio-based `async` tasks spawner.
#[derive(Copy, Clone, Debug)]
pub struct RuntimeTokio;

#[async_trait::async_trait]
impl Runtime for RuntimeTokio {
    fn spawn<R>(&self, future: impl futures::Future<Output = R> + Send + 'static)
    where
        R: Send + 'static,
    {
        tokio::spawn(future);
    }

    async fn sleep(self, delay: u64) {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await
    }

    async fn sleep_microseconds(self, delay: u64) {
        tokio::time::sleep(tokio::time::Duration::from_micros(delay)).await
    }
}
