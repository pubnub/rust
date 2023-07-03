//! # Futures implementation using Tokio runtime
//!
//! This module contains [`TokioSpawner`] type.
//!
//! It requires the [`future_tokio` feature] to be enabled.
//!
//! [`future_tokio` feature]: ../index.html#features

use futures::future::FutureObj;
use futures::task::{Spawn, SpawnError};

/// Tokio-based `async` tasks spawner.
pub struct TokioSpawner;

impl Spawn for TokioSpawner {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        tokio::task::spawn(future);
        Ok(())
    }
}
