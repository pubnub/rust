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
    executor: &Arc<SubscribeEffectExecutor>,
) -> BoxFuture<'static, Result<Vec<SubscribeEvent>, PubNubError>> {
    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    executor(
        None,
        SubscriptionParams {
            channels: &channels,
            channel_groups: &channel_groups,
            attempt,
            reason: Some(reason),
            effect_id: &effect_id,
        },
    )
    .map(|result| {
        result
            .map(|subscribe_result| {
                vec![SubscribeEvent::HandshakeSuccess {
                    cursor: subscribe_result.cursor,
                }]
            })
            .or_else(|error| {
                Ok(vec![SubscribeEvent::HandshakeFailure {
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
    async fn initialize_handshake_reconnect_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |cursor, params| {
                assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(params.attempt, 1);
                assert_eq!(cursor, None);
                assert_eq!(
                    params.reason.unwrap(),
                    PubNubError::Transport {
                        details: "test".into(),
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
            },
            "id",
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::HandshakeSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_handskahe_failure_event_on_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_, _| {
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
            1,
            PubNubError::Transport {
                details: "test".into(),
            },
            "id",
            &mock_handshake_function,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            SubscribeEvent::HandshakeFailure { .. }
        ));
    }
}
