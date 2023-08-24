use crate::{
    core::{PubNubError, RequestRetryPolicy},
    dx::subscribe::event_engine::{
        effects::SubscribeEffectExecutor, types::SubscriptionParams, SubscribeEvent,
    },
    lib::alloc::{string::String, sync::Arc, vec, vec::Vec},
};
use futures::TryFutureExt;
use log::info;

pub(super) async fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: PubNubError,
    effect_id: &str,
    retry_policy: &RequestRetryPolicy,
    executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    if !retry_policy.retriable(&attempt, Some(&reason)) {
        return vec![SubscribeEvent::HandshakeReconnectGiveUp { reason }];
    }

    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    executor(SubscriptionParams {
        channels,
        channel_groups,
        cursor: None,
        attempt,
        reason: Some(reason),
        effect_id,
    })
    .map_ok_or_else(
        |error| {
            log::error!("Handshake reconnection error: {:?}", error);

            (!matches!(error, PubNubError::EffectCanceled))
                .then(|| vec![SubscribeEvent::HandshakeReconnectFailure { reason: error }])
                .unwrap_or(vec![])
        },
        |subscribe_result| {
            vec![SubscribeEvent::HandshakeReconnectSuccess {
                cursor: subscribe_result.cursor,
            }]
        },
    )
    .await
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[tokio::test]
    async fn initialize_handshake_reconnect_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |params| {
            assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(params.attempt, 1);
            assert_eq!(params.cursor, None);
            assert_eq!(
                params.reason.unwrap(),
                PubNubError::Transport {
                    details: "test".into(),
                    response: None
                }
            );
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
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            1,
            PubNubError::Transport {
                details: "test".into(),
                response: None,
            },
            "id",
            &RequestRetryPolicy::Linear {
                delay: 0,
                max_retry: 1,
            },
            &mock_handshake_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::HandshakeReconnectSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_handshake_reconnect_failure_event_on_err() {
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
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            1,
            PubNubError::Transport {
                details: "test".into(),
                response: None,
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_handshake_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::HandshakeReconnectGiveUp { .. }
        ));
    }
}
