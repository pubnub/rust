use futures::TryFutureExt;
use log::info;

use crate::core::PubNubError;
use crate::subscribe::SubscriptionCursor;
use crate::{
    dx::subscribe::event_engine::{
        effects::SubscribeEffectExecutor, SubscribeEvent, SubscriptionInput, SubscriptionParams,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};

pub(super) async fn execute(
    input: &SubscriptionInput,
    cursor: &Option<SubscriptionCursor>,
    effect_id: &str,
    executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!(
        "Handshake for\nchannels: {:?}\nchannel groups: {:?}",
        input.channels().unwrap_or_default(),
        input.channel_groups().unwrap_or_default()
    );

    if input.is_empty {
        return vec![SubscribeEvent::UnsubscribeAll];
    }

    executor(SubscriptionParams {
        channels: &input.channels(),
        channel_groups: &input.channel_groups(),
        cursor: None,
        attempt: 0,
        reason: None,
        effect_id,
    })
    .map_ok_or_else(
        |error| {
            log::error!("Handshake error: {:?}", error);

            // Cancel is possible and no retries should be done.
            (!matches!(error, PubNubError::EffectCanceled))
                .then(|| vec![SubscribeEvent::HandshakeFailure { reason: error }])
                .unwrap_or(vec![])
        },
        |subscribe_result| {
            let cursor = {
                if cursor.is_none() {
                    subscribe_result.cursor
                } else {
                    let mut cursor = cursor.clone().unwrap_or_default();
                    cursor.region = subscribe_result.cursor.region;
                    cursor
                }
            };
            vec![SubscribeEvent::HandshakeSuccess { cursor }]
        },
    )
    .await
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::SubscribeResult};
    use futures::FutureExt;

    #[tokio::test]
    async fn initialize_handshake_for_first_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |params| {
            assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(params.cursor, None);
            assert_eq!(params.attempt, 0);
            assert_eq!(params.reason, None);
            assert_eq!(params.effect_id, "id");

            async move {
                Ok(SubscribeResult {
                    cursor: Default::default(),
                    messages: vec![],
                })
            }
            .boxed()
        });

        let result = execute(
            &SubscriptionInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &None,
            "id",
            &mock_handshake_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::HandshakeSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_handshake_failure_event_on_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    response: None,
                })
            }
            .boxed()
        });

        let result = execute(
            &SubscriptionInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &None,
            "id",
            &mock_handshake_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::HandshakeFailure { .. }
        ));
    }

    #[tokio::test]
    async fn return_empty_event_on_effect_cancel_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |_| async move { Err(PubNubError::EffectCanceled) }.boxed());

        let result = execute(
            &SubscriptionInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &None,
            "id",
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_empty());
    }
}
