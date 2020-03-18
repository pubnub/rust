//! [`Runtime`] mocks.

use crate::Runtime;
use std::future::Future;
use std::pin::Pin;

use mockall::mock;

mod gen {
    #![allow(missing_docs)]
    use super::{mock, Future, Pin};

    mock! {
        pub Runtime {
            fn mock_workaround_spawn<O: 'static>(&self, future: Pin<Box<dyn Future<Output = O> + Send + 'static>>) {}
        }
        trait Clone {
            fn clone(&self) -> Self;
        }
    }
}
pub use gen::*;

impl std::fmt::Debug for MockRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockRuntime").finish()
    }
}

impl Runtime for MockRuntime {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.mock_workaround_spawn(Box::pin(future))
    }
}
