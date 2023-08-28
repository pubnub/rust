//! This module contains the task spawning trait used in the PubNub client.
//!
//! The [`Spawner`] trait is used to spawn async tasks in work of the PubNub
//! client.

use crate::lib::{
    alloc::{
        fmt::{Debug, Formatter, Result},
        sync::Arc,
    },
    core::future::Future,
};
use futures::future::{BoxFuture, FutureExt};

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

#[derive(Clone)]
pub(crate) struct RuntimeSupport {
    spawner: Arc<dyn Fn(BoxFuture<'static, ()>) + Send + Sync>,
    sleeper: Arc<dyn Fn(u64) -> BoxFuture<'static, ()> + Send + Sync>,
}

impl RuntimeSupport {
    pub fn new<R>(runtime: Arc<R>) -> Self
    where
        R: Runtime + Copy + Send + Sync + 'static,
    {
        let spawn_runtime = runtime.clone();
        let sleep_runtime = runtime.clone();

        Self {
            sleeper: Arc::new(move |delay| sleep_runtime.sleep(delay).boxed()),
            spawner: Arc::new(Box::new(move |future| {
                spawn_runtime.spawn(future);
            })),
        }
    }
}

#[async_trait::async_trait]
impl Runtime for RuntimeSupport {
    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static)
    where
        R: Send + 'static,
    {
        (self.spawner.clone())(
            async move {
                future.await;
            }
            .boxed(),
        );
    }

    async fn sleep(self, delay: u64) {
        (self.sleeper)(delay).await
    }
}

impl Debug for RuntimeSupport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "RuntimeSupport {{}}")
    }
}
