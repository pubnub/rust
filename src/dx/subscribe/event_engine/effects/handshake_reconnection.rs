use futures::TryFutureExt;
use log::info;

use crate::{
    core::{PubNubError, RequestRetryConfiguration},
    dx::subscribe::{
        event_engine::{
            effects::SubscribeEffectExecutor, SubscribeEvent, SubscriptionInput, SubscriptionParams,
        },
        SubscriptionCursor,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};

pub(super) async fn execute(
    input: &SubscriptionInput,
    cursor: &Option<SubscriptionCursor>,
    attempt: u8,
    reason: PubNubError,
    effect_id: &str,
    retry_policy: &RequestRetryConfiguration,
    executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    if !matches!(reason, PubNubError::EffectCanceled)
        && !retry_policy.retriable(Some("/v2/subscribe"), &attempt, Some(&reason))
    {
        return vec![SubscribeEvent::HandshakeReconnectGiveUp { reason }];
    }

    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
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
            let cursor = {
                if cursor.is_none() {
                    subscribe_result.cursor
                } else {
                    let mut cursor = cursor.clone().unwrap_or_default();
                    cursor.region = subscribe_result.cursor.region;
                    cursor
                }
            };
            vec![SubscribeEvent::HandshakeReconnectSuccess { cursor }]
        },
    )
    .await
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::TransportResponse;
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[tokio::test]
    async fn initialize_handshake_reconnect_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |params| {
            assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(params.cursor, None);
            assert_eq!(params.attempt, 1);
            assert_eq!(
                params.reason.unwrap(),
                PubNubError::Transport {
                    details: "test".into(),
                    response: Some(Box::new(TransportResponse {
                        status: 500,
                        ..Default::default()
                    }))
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
            &SubscriptionInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &None,
            1,
            PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            },
            "id",
            &RequestRetryConfiguration::Linear {
                delay: 0,
                max_retry: 1,
                excluded_endpoints: None,
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
    async fn return_handshake_reconnect_give_up_event_on_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    response: Some(Box::new(TransportResponse {
                        status: 500,
                        ..Default::default()
                    })),
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
            11,
            PubNubError::Transport {
                details: "test".into(),
                response: None,
            },
            "id",
            &RequestRetryConfiguration::Linear {
                max_retry: 10,
                delay: 0,
                excluded_endpoints: None,
            },
            &mock_handshake_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::HandshakeReconnectGiveUp { .. }
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
            1,
            PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            },
            "id",
            &RequestRetryConfiguration::Linear {
                delay: 0,
                max_retry: 1,
                excluded_endpoints: None,
            },
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn return_handshake_reconnect_give_up_event_on_err_with_none_auto_retry_policy() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    response: Some(Box::new(TransportResponse {
                        status: 500,
                        ..Default::default()
                    })),
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
            1,
            PubNubError::Transport {
                details: "test".into(),
                response: None,
            },
            "id",
            &RequestRetryConfiguration::None,
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
