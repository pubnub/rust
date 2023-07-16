use crate::{
    core::{PubNubError, RequestRetryPolicy},
    dx::subscribe::{
        event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
        SubscribeCursor,
    },
    lib::alloc::{string::String, sync::Arc, vec, vec::Vec},
};
use log::info;

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    _attempt: u8,
    _reason: PubNubError,
    _effect_id: &str,
    retry_policy: &RequestRetryPolicy,
    _executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!(
        "Receive reconnection at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );
    let _retry_policy = retry_policy.clone();

    // TODO: If retriable (`std` environment) we need to delay next call to the PubNub.

    //    executor(SubscriptionParams {
    //        channels: &channels,
    //        channel_groups: &channel_groups,
    //        cursor: Some(cursor),
    //        attempt,
    //        reason: Some(reason),
    //        effect_id: &effect_id,
    //    })
    //    .map(move |result| {
    //        result
    //            .map(|subscribe_result| {
    //                vec![SubscribeEvent::ReceiveReconnectSuccess {
    //                    cursor: subscribe_result.cursor,
    //                    messages: subscribe_result.messages,
    //                }]
    //            })
    //            .or_else(|error| {
    //                Ok(match error {
    //                    PubNubError::Transport { status, .. } | PubNubError::API { status, .. }
    //                        if !retry_policy.retriable(attempt, status) =>
    //                    {
    //                        vec![SubscribeEvent::ReceiveReconnectGiveUp { reason: error }]
    //                    }
    //                    _ if !matches!(error, PubNubError::EffectCanceled) => {
    //                        vec![SubscribeEvent::ReceiveReconnectFailure { reason: error }]
    //                    }
    //                    _ => vec![],
    //                })
    //            })
    //    })
    //    .boxed()
    vec![]
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
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
                    status: 500
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
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
                status: 500,
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_receive_function,
        );

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
                    status: 500,
                })
            }
            .boxed()
        });

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
                status: 500,
            },
            "id",
            &RequestRetryPolicy::None,
            &mock_receive_function,
        );

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveReconnectFailure { .. }
        ));
    }
}
