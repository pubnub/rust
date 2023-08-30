use futures::TryFutureExt;
use log::info;

use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{
            effects::SubscribeEffectExecutor, types::SubscriptionParams, SubscribeEvent,
            SubscribeInput,
        },
        SubscribeCursor,
    },
    lib::alloc::{sync::Arc, vec, vec::Vec},
};

pub(crate) async fn execute(
    input: &SubscribeInput,
    cursor: &SubscribeCursor,
    effect_id: &str,
    executor: &Arc<SubscribeEffectExecutor>,
) -> Vec<SubscribeEvent> {
    info!(
        "Receive at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        input.channels().unwrap_or(Vec::new()),
        input.channel_groups().unwrap_or(Vec::new())
    );

    if input.is_empty {
        return vec![SubscribeEvent::UnsubscribeAll];
    }

    executor(SubscriptionParams {
        channels: &input.channels(),
        channel_groups: &input.channel_groups(),
        cursor: Some(cursor),
        attempt: 0,
        reason: None,
        effect_id,
    })
    .map_ok_or_else(
        |error| {
            log::error!("Receive error: {:?}", error);

            (!matches!(error, PubNubError::EffectCanceled))
                .then(|| vec![SubscribeEvent::ReceiveFailure { reason: error }])
                .unwrap_or(vec![])
        },
        |subscribe_result| {
            vec![SubscribeEvent::ReceiveSuccess {
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
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[tokio::test]
    async fn receive_messages() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |params| {
            assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(params.attempt, 0);
            assert_eq!(params.reason, None);
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
            "id",
            &mock_receive_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveSuccess { .. }
        ));
    }

    #[tokio::test]
    async fn return_receive_failure_event_on_err() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    response: None,
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
            "id",
            &mock_receive_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            SubscribeEvent::ReceiveFailure { .. }
        ));
    }
}
