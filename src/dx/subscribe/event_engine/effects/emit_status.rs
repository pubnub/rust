use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::EmitStatusEffectExecutor, SubscribeEvent},
        SubscribeStatus,
    },
    lib::alloc::sync::Arc,
};
use futures::{future::BoxFuture, FutureExt};
use log::info;

pub(super) fn execute(
    status: SubscribeStatus,
    executor: &Arc<EmitStatusEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!("Emit status: {status:?}");
    let cloned_executor = executor.clone();

    //    async move {
    //        cloned_executor(status);
    //
    //        Ok(vec![])
    //    }
    //    .boxed()
    vec![]
}

#[cfg(test)]
mod should {
    use super::*;

    #[tokio::test]
    async fn emit_expected_status() {
        let mut function_called = false;
        let emit_status_function: Arc<EmitStatusEffectExecutor> = Arc::new(|status| {
            assert!(matches!(status, SubscribeStatus::Connected));
            function_called = true;
        });

        execute(SubscribeStatus::Connected, &emit_status_function)
            .await
            .expect("expected result");

        assert!(function_called);
    }
}
