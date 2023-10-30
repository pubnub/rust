use futures::TryFutureExt;
use log::info;

use crate::{
    core::{PubNubError, RequestRetryPolicy},
    dx::subscribe::{
        event_engine::{
            effects::SubscribeEffectExecutor, types::SubscriptionParams, SubscribeEvent,
            SubscribeInput,
        },
        SubscribeCursor,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute(
    input: &SubscribeInput,
    cursor: &SubscribeCursor,
    attempt: u8,
    reason: PubNubError,
    effect_id: &str,
    retry_policy: &RequestRetryPolicy,
    executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    if !retry_policy.retriable(&attempt, Some(&reason)) {
        return vec![SubscribeEvent::ReceiveReconnectGiveUp { reason }];
    }

    info!(
        "Receive reconnection at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        input.channels().unwrap_or_default(),
        input.channel_groups().unwrap_or_default()
    );

    if input.is_empty {
        return vec![SubscribeEvent::UnsubscribeAll];
    }

    executor(SubscriptionParams {
        channels: &input.channels(),
        channel_groups: &input.channel_groups(),
        cursor: Some(cursor),
        attempt,
        reason: Some(reason),
        effect_id,
    })
    .map_ok_or_else(
        |error| {
            log::debug!("Receive reconnection error: {:?}", error);

            (!matches!(error, PubNubError::EffectCanceled))
                .then(|| vec![SubscribeEvent::ReceiveReconnectFailure { reason: error }])
                .unwrap_or(vec![])
        },
        |subscribe_result| {
            vec![SubscribeEvent::ReceiveReconnectSuccess {
                cursor: subscribe_result.cursor,
                messages: subscribe_result.messages,
            }]
        },
    )
    .await
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::{PubNubError, TransportResponse},
        dx::subscribe::result::SubscribeResult,
        lib::alloc::boxed::Box,
    };
    use futures::FutureExt;

    #[tokio::test]
    async fn receive_reconnect() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |params| {
            assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(params.attempt, 10);
            assert_eq!(
                params.reason,
                Some(PubNubError::Transport {
                    details: "test".into(),
                    response: Some(Box::new(TransportResponse {
                        status: 500,
                        ..Default::default()
                    })),
                })
            );
            assert_eq!(params.cursor, Some(&Default::default()));
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
            &SubscribeInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_receive_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveReconnectSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_receive_reconnect_failure_event_on_err() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
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
            &SubscribeInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &Default::default(),
            5,
            PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_receive_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveReconnectFailure { .. }
        ));
    }

    #[tokio::test]
    async fn return_receive_reconnect_give_up_event_on_err() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
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
            &SubscribeInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            },
            "id",
            &RequestRetryPolicy::Linear {
                delay: 0,
                max_retry: 1,
            },
            &mock_receive_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveReconnectGiveUp { .. }
        ));
    }
}
