//! Module with `Wait` effect implementation.

use crate::{
    lib::alloc::{sync::Arc, vec, vec::Vec},
    presence::event_engine::{effects::WaitEffectExecutor, PresenceEvent},
};

use futures::TryFutureExt;
use log::info;

pub(super) async fn execute(
    effect_id: &str,
    executor: &Arc<WaitEffectExecutor>,
) -> Vec<PresenceEvent> {
    info!("Heartbeat cooling down");

    executor(effect_id)
        .map_ok_or_else(|_| vec![], |_| vec![PresenceEvent::TimesUp])
        .await
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::core::PubNubError;
    use futures::{future::ready, FutureExt};

    #[tokio::test]
    async fn return_time_up_event() {
        let mock_wait_function: Arc<WaitEffectExecutor> = Arc::new(move |effect_id| {
            assert_eq!(effect_id, "id");
            ready(Ok(())).boxed()
        });

        let result = execute("id", &mock_wait_function).await;

        assert!(!result.is_empty());
        assert!(matches!(result.first().unwrap(), PresenceEvent::TimesUp));
    }

    #[tokio::test]
    async fn return_empty_list_on_cancel() {
        let mock_wait_function: Arc<WaitEffectExecutor> = Arc::new(move |effect_id| {
            assert_eq!(effect_id, "id");
            ready(Err(PubNubError::EffectCanceled)).boxed()
        });

        let result = execute("id", &mock_wait_function).await;

        assert!(result.is_empty());
    }
}
