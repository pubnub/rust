use crate::{
    dx::subscribe::event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use log::info;

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    effect_id: &str,
    _executor: &Arc<SubscribeEffectExecutor>,
) -> Option<Vec<SubscribeEvent>> {
    info!(
        "Handshake for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new())
    );

    // let result: Result<SubscribeResult, PubNubError> = executor(channels, channel_groups, 0, None);
    //
    // Some(result.unwrap_or_else(|err| vec![SubscribeEvent::HandshakeFailure { reason: err }]))
    None
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::SubscribeResult};
    use futures::FutureExt;

    #[tokio::test]
    async fn initialize_handshake_for_first_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |cursor, params| {
                assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(params._attempt, 0);
                assert_eq!(cursor, None);
                assert_eq!(params._reason, None);
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
            "id",
            &mock_handshake_function,
        );

        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            &SubscribeEvent::HandshakeSuccess { .. }
        ))
    }

    #[tokio::test]
    async fn return_handshake_failure_event_on_err() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_, _| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                })
            }
            .boxed()
        });

        let binding = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            "id",
            &mock_handshake_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(result, &SubscribeEvent::HandshakeFailure { .. }));
    }
}
