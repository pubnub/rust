use crate::{
    core::PubNubError,
    dx::subscribe::event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use log::info;

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    _attempt: u8,
    _reason: PubNubError,
    _executor: &Arc<SubscribeEffectExecutor>,
) -> Option<Vec<SubscribeEvent>> {
    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    // let result: Result<Vec<SubscribeEvent>, PubNubError> =
    //     executor(channels, channel_groups, attempt, Some(reason));
    // Some(
    //     result
    //         .unwrap_or_else(|err| vec![SubscribeEvent::HandshakeReconnectFailure { reason: err }]),
    // )
    None
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[test]
    fn initialize_handshake_reconnect_attempt() {
        let mock_handshake_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |cursor, params| {
                assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(params._attempt, 1);
                assert_eq!(cursor, None);
                assert_eq!(
                    params._reason.unwrap(),
                    PubNubError::Transport {
                        details: "test".into(),
                    }
                );

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
            &mock_handshake_function,
        );

        assert!(result.is_some());
        assert!(!result.unwrap().is_empty())
    }

    #[test]
    fn return_handskahe_failure_event_on_err() {
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
            1,
            PubNubError::Transport {
                details: "test".into(),
            },
            &mock_handshake_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(
            result,
            &SubscribeEvent::HandshakeReconnectFailure { .. }
        ));
    }
}
