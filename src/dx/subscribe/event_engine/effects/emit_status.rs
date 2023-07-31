use crate::{
    dx::subscribe::{
        event_engine::{effects::EmitStatusEffectExecutor, SubscribeEvent},
        SubscribeStatus,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};
use log::info;

pub(super) async fn execute(
    status: SubscribeStatus,
    executor: &Arc<EmitStatusEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!("Emit status: {status:?}");

    executor(status);

    vec![]
}

#[cfg(test)]
mod should {
    use super::*;

    #[tokio::test]
    async fn emit_expected_status() {
        let emit_status_function: Arc<EmitStatusEffectExecutor> = Arc::new(|status| {
            assert!(matches!(status, SubscribeStatus::Connected));
        });

        execute(SubscribeStatus::Connected, &emit_status_function).await;
    }
}
