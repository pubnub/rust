//! This module contains the task spawning trait used in the PubNub client.
//!
//! The [`Spawner`] trait is used to spawn async tasks in work of the PubNub
//! client.

use crate::lib::core::future::Future;

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
/// impl Runtime for MyRuntime {
///    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
///       // spawn the Future
///       // e.g. tokio::spawn(future);
///    }
/// }
/// ```
pub trait Runtime: Clone + Send {
    /// Spawn a task.
    ///
    /// This method is used to spawn a task.
    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static)
    where
        R: Send + 'static;
}
