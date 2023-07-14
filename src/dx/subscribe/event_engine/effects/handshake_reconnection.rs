use crate::core::RequestRetryPolicy;
use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
        SubscriptionParams,
    },
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use futures::{future::BoxFuture, FutureExt};
use log::info;

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: PubNubError,
    effect_id: &str,
    retry_policy: &RequestRetryPolicy,
    executor: &Arc<SubscribeEffectExecutor>,
) -> BoxFuture<'static, Result<Vec<SubscribeEvent>, PubNubError>> {
    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );
    let retry_policy = retry_policy.clone();

    // TODO: If retriable (`std` environment) we need to delay next call to the PubNub.

    executor(SubscriptionParams {
        channels: &channels,
        channel_groups: &channel_groups,
        cursor: None,
        attempt,
        reason: Some(reason),
        effect_id: &effect_id,
    })
    .map(move |result| {
        result
            .map(|subscribe_result| {
                vec![SubscribeEvent::HandshakeReconnectSuccess {
                    cursor: subscribe_result.cursor,
                }]
            })
            .or_else(|error| {
                Ok(match error {
                    PubNubError::Transport { status, .. } | PubNubError::API { status, .. }
                        if !retry_policy.retriable(attempt, status) =>
                    {
                        vec![SubscribeEvent::HandshakeReconnectGiveUp { reason: error }]
                    }
                    _ if !matches!(error, PubNubError::EffectCanceled) => {
                        vec![SubscribeEvent::HandshakeReconnectFailure { reason: error }]
                    }
                    _ => vec![],
                })
            })
    })
    .boxed()
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
                    status: 500
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
                status: 500,
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::HandshakeReconnectSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_handskahe_failure_event_on_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    status: 500,
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
                status: 500,
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::HandshakeReconnectFailure { .. }
        ));
    }
}
