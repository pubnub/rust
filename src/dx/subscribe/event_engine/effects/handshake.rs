use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::HandshakeEffectExecutor, SubscribeEvent},
        SubscribeResult,
    },
    lib::alloc::{string::String, vec, vec::Vec},
};
use log::info;
use std::sync::Arc;

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    executor: &Arc<HandshakeEffectExecutor>,
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
    use crate::core::PubNubError;

    #[test]
    fn initialize_handshake_for_first_attempt() {
        fn mock_handshake_function(
            channels: &Option<Vec<String>>,
            channel_groups: &Option<Vec<String>>,
            attempt: u8,
            reason: Option<PubNubError>,
        ) -> Result<Vec<SubscribeEvent>, PubNubError> {
            assert_eq!(channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(attempt, 0);
            assert_eq!(reason, None);

            Ok(vec![SubscribeEvent::HandshakeSuccess {
                cursor: Default::default(),
            }])
        }

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            mock_handshake_function,
        );

        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().first().unwrap(),
            &SubscribeEvent::HandshakeSuccess { .. }
        ))
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
            mock_handshake_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(result, &SubscribeEvent::HandshakeFailure { .. }));
    }
}
