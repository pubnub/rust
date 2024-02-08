//! Heartbeat effect module.
//!
//! Module contains implementation of `Heartbeat` and `Delayed heartbeat` effect
//! which is used to announce `user_id` presence on specified channels and
//! groups.

use futures::TryFutureExt;
use log::info;

use crate::{
    core::{PubNubError, RequestRetryConfiguration},
    lib::alloc::{sync::Arc, vec, vec::Vec},
    presence::event_engine::{
        effects::HeartbeatEffectExecutor, PresenceEvent, PresenceInput, PresenceParameters,
    },
};

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute(
    input: &PresenceInput,
    attempt: u8,
    reason: Option<PubNubError>,
    effect_id: &str,
    retry_policy: &RequestRetryConfiguration,
    executor: &Arc<HeartbeatEffectExecutor>,
) -> Vec<PresenceEvent> {
    if let Some(reason) = reason.clone() {
        if !retry_policy.retriable(Some("/v2/presence"), &attempt, Some(&reason)) {
            return vec![PresenceEvent::HeartbeatGiveUp { reason }];
        }
    }

    let channel_groups: Option<Vec<String>> = input.channel_groups();
    let channels: Option<Vec<String>> = input.channels();

    info!(
        "Heartbeat for\nchannels: {:?}\nchannel groups: {:?}",
        channels, channel_groups
    );

    executor(PresenceParameters {
        channels: &channels,
        channel_groups: &channel_groups,
        attempt,
        reason,
        effect_id,
    })
    .map_ok_or_else(
        |error| {
            log::error!("Handshake error: {:?}", error);

            // Cancel is possible and no retries should be done.
            (!matches!(error, PubNubError::EffectCanceled))
                .then(|| vec![PresenceEvent::HeartbeatFailure { reason: error }])
                .unwrap_or(vec![])
        },
        |_| vec![PresenceEvent::HeartbeatSuccess],
    )
    .await
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{
        core::{PubNubError, TransportResponse},
        dx::presence::HeartbeatResult,
    };
    use futures::FutureExt;

    #[tokio::test]
    async fn return_heartbeat_success_event() {
        let mocked_heartbeat_function: Arc<HeartbeatEffectExecutor> = Arc::new(move |parameters| {
            assert_eq!(parameters.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(parameters.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(parameters.attempt, 0);
            assert_eq!(parameters.reason, None);
            assert_eq!(parameters.effect_id, "id");

            async move { Ok(HeartbeatResult) }.boxed()
        });

        let result = execute(
            &PresenceInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            0,
            None,
            "id",
            &RequestRetryConfiguration::None,
            &mocked_heartbeat_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            PresenceEvent::HeartbeatSuccess
        ));
    }

    #[tokio::test]
    async fn return_heartbeat_failed_event_on_error() {
        let mocked_heartbeat_function: Arc<HeartbeatEffectExecutor> = Arc::new(move |_| {
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
            &PresenceInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            0,
            Some(PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            }),
            "id",
            &RequestRetryConfiguration::Linear {
                max_retry: 5,
                delay: 2,
                excluded_endpoints: None,
            },
            &mocked_heartbeat_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            PresenceEvent::HeartbeatFailure { .. }
        ));
    }

    #[tokio::test]
    async fn return_heartbeat_give_up_event_on_error() {
        let mocked_heartbeat_function: Arc<HeartbeatEffectExecutor> = Arc::new(move |_| {
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
            &PresenceInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            5,
            Some(PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            }),
            "id",
            &RequestRetryConfiguration::Linear {
                delay: 0,
                max_retry: 1,
                excluded_endpoints: None,
            },
            &mocked_heartbeat_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            PresenceEvent::HeartbeatGiveUp { .. }
        ));
    }

    #[tokio::test]
    async fn return_empty_event_on_effect_cancel_err() {
        let mocked_heartbeat_function: Arc<HeartbeatEffectExecutor> =
            Arc::new(move |_| async move { Err(PubNubError::EffectCanceled) }.boxed());

        let result = execute(
            &PresenceInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            5,
            Some(PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            }),
            "id",
            &RequestRetryConfiguration::Linear {
                max_retry: 5,
                delay: 2,
                excluded_endpoints: None,
            },
            &mocked_heartbeat_function,
        )
        .await;

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn return_heartbeat_give_up_event_on_error_with_none_auto_retry_policy() {
        let mocked_heartbeat_function: Arc<HeartbeatEffectExecutor> = Arc::new(move |_| {
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
            &PresenceInput::new(
                &Some(vec!["ch1".to_string()]),
                &Some(vec!["cg1".to_string()]),
            ),
            5,
            Some(PubNubError::Transport {
                details: "test".into(),
                response: Some(Box::new(TransportResponse {
                    status: 500,
                    ..Default::default()
                })),
            }),
            "id",
            &RequestRetryConfiguration::Linear {
                delay: 0,
                max_retry: 1,
                excluded_endpoints: None,
            },
            &mocked_heartbeat_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            PresenceEvent::HeartbeatGiveUp { .. }
        ));
    }
}
