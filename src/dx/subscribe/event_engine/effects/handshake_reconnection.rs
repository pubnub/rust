use crate::lib::alloc::{string::String, vec, vec::Vec};
use crate::{
    core::PubNubError,
    dx::subscribe::event_engine::{effect_handler::HandshakeFunction, SubscribeEvent},
};

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    attempt: u8,
    reason: PubNubError,
    executor: HandshakeFunction,
) -> Option<Vec<SubscribeEvent>> {
    Some(
        executor(channels, channel_groups, attempt, Some(reason)).unwrap_or_else(|err| {
            vec![SubscribeEvent::HandshakeReconnectFailure {
                reason: err,
                attempts: attempt + 1,
            }]
        }),
    )
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::SubscribeCursor};

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
                cursor: SubscribeCursor {
                    timetoken: 0,
                    region: 0,
                },
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

        let attempt = 10;
        let expected_attempts = attempt + 1;

        let binding = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            attempt,
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
        match result {
            SubscribeEvent::HandshakeReconnectFailure { attempts, .. } => {
                assert_eq!(*attempts, expected_attempts);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
