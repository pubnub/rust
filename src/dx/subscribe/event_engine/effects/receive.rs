use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
        SubscribeCursor, SubscriptionParams,
    },
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use futures::{future::BoxFuture, FutureExt};
use log::info;

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    effect_id: &str,
    executor: &Arc<SubscribeEffectExecutor>,
) -> BoxFuture<'static, Result<Vec<SubscribeEvent>, PubNubError>> {
    info!(
        "Receive at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    executor(
        Some(&cursor),
        SubscriptionParams {
            channels: &channels,
            channel_groups: &channel_groups,
            cursor: Some(cursor),
            attempt: 0,
            reason: None,
            effect_id: &effect_id,
        },
    )
    .map(|result| {
        result
            .map(|subscribe_result| {
                vec![SubscribeEvent::ReceiveSuccess {
                    cursor: subscribe_result.cursor,
                    messages: subscribe_result.messages,
                }]
            })
            .or_else(|error| {
                Ok(vec![SubscribeEvent::ReceiveFailure {
                    reason: error.into(),
                }])
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
    async fn receive_messages() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |cursor, params| {
                assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(params.attempt, 0);
                assert_eq!(params.reason, None);
                assert_eq!(cursor, Some(&Default::default()));
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
            "id",
            &mock_receive_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::ReceiveSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_handshake_failure_event_on_err() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_, _| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                })
            }
            .boxed()
        });

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            &Default::default(),
            "id",
            &mock_receive_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::ReceiveFailure { .. }
        ));
    }
}
