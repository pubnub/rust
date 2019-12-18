use crate::Runtime;
use std::future::Future;

use mockall::mock;

mock! {
    pub Runtime {}
    trait Clone {
        fn clone(&self) -> Self {}
    }
    trait Runtime: Clone + Send + Sync + Unpin + Debug {
        fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + Send + 'static {}
    }
}

impl std::fmt::Debug for MockRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockRuntime").finish()
    }
}
