use crate::{
    dx::subscribe::{
        event_engine::{effects::EmitStatusEffectExecutor, SubscribeEvent},
        ConnectionStatus,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};
use log::info;

pub(super) async fn execute(
    status: ConnectionStatus,
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
            assert!(matches!(status, ConnectionStatus::Connected));
        });

        execute(ConnectionStatus::Connected, &emit_status_function).await;
    }
}
