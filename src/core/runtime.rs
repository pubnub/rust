use super::PubNubError;
use crate::lib::core::future::Future;

/// This module contains the task spawning trait used in the PubNub client.
///
/// The [`Spawner`] trait is used to spawn async tasks in work of the PubNub
/// client.

/// PubNub spawner trait.
///
/// This trait is used to spawn async tasks in work of the PubNub client.
/// It is used to spawn tasks for the proper work of some features
/// that require async tasks to be spawned.
///
/// # Examples
/// ```
/// use pubnub_core::core::spawner::Spawner;
/// use crate::lib::core::future::Future;
/// use super::PubNubError;
///
/// struct MySpawner;
///
/// impl Spawner for MySpawner {
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
