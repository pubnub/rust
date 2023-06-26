use crate::{
    core::PubNubError,
    dx::subscribe::event_engine::{effects::HandshakeEffectExecutor, SubscribeEvent},
    lib::alloc::{string::String, vec, vec::Vec},
};
use log::info;

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: PubNubError,
    executor: &HandshakeEffectExecutor,
) -> Option<Vec<SubscribeEvent>> {
    info!(
        "Handshake reconnection for\nchannels: {:?}\nchannel groups: {:?}",
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    let result: Result<Vec<SubscribeEvent>, PubNubError> =
        executor(channels, channel_groups, attempt, Some(reason));
    Some(
        result
            .unwrap_or_else(|err| vec![SubscribeEvent::HandshakeReconnectFailure { reason: err }]),
    )
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::PubNubError;

    #[test]
    fn initialize_handshake_reconnect_attempt() {
        fn mock_handshake_function(
            channels: &Option<Vec<String>>,
            channel_groups: &Option<Vec<String>>,
            attempt: u8,
            reason: Option<PubNubError>,
        ) -> Result<Vec<SubscribeEvent>, PubNubError> {
            assert_eq!(channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(attempt, 1);
            assert_eq!(
                reason.unwrap(),
                PubNubError::Transport {
                    details: "test".into(),
                }
            );

            Ok(vec![SubscribeEvent::HandshakeSuccess {
                cursor: Default::default(),
            }])
        }

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            1,
            PubNubError::Transport {
                details: "test".into(),
            },
            mock_handshake_function,
        );

        assert!(result.is_some());
        assert!(!result.unwrap().is_empty())
    }

    #[test]
    fn return_handskahe_failure_event_on_err() {
        fn mock_handshake_function(
            _channels: &Option<Vec<String>>,
            _channel_groups: &Option<Vec<String>>,
            _attempt: u8,
            _reason: Option<PubNubError>,
        ) -> Result<Vec<SubscribeEvent>, PubNubError> {
            Err(PubNubError::Transport {
                details: "test".into(),
            })
        }

        let binding = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            1,
            PubNubError::Transport {
                details: "test".into(),
            },
            mock_handshake_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(
            result,
            &SubscribeEvent::HandshakeReconnectFailure { .. }
        ));
    }
}
