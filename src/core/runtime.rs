//! This module contains the task spawning trait used in the PubNub client.
//!
//! The [`Spawner`] trait is used to spawn async tasks in work of the PubNub
//! client.

use crate::lib::{alloc::boxed::Box, core::future::Future};

/// PubNub spawner trait.
///
/// This trait is used to spawn async tasks in work of the PubNub client.
/// It is used to spawn tasks for the proper work of some features
/// that require async tasks to be spawned.
///
/// # Examples
/// ```
/// use pubnub::core::{Runtime, PubNubError};
/// use std::future::Future;
///
/// #[derive(Clone)]
/// struct MyRuntime;
///
/// #[async_trait::async_trait]
/// impl Runtime for MyRuntime {
///    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
///       // spawn the Future
///       // e.g. tokio::spawn(future);
///    }
///    
///    async fn sleep(self, _delay: u64) {
///       // e.g. tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await
///    }
/// }
/// ```
#[async_trait::async_trait]
pub trait Runtime: Clone + Send {
    /// Spawn a task.
    ///
    /// This method is used to spawn a task.
    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static)
    where
        R: Send + 'static;

    /// Put current task to "sleep".
    ///
    /// Sleep current task for specified amount of time (in seconds).
    async fn sleep(self, delay: u64);
}
