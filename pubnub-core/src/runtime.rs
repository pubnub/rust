use std::fmt::Debug;
use std::future::Future;

/// Runtime abstracts away the underlying runtime we use for task scheduling.
pub trait Runtime: Clone + Send + Sync + Unpin + Debug {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;
}
