use crate::dx::subscribe::event_engine::{effect_handler::HandshakeFunction, SubscribeEvent};

pub(super) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    executor: HandshakeFunction,
) -> Option<Vec<SubscribeEvent>> {
    Some(vec![executor(channels, channel_groups, 0, None)])
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::SubscribeCursor};

    #[test]
    fn initialize_handshake_for_first_attempt() {
        fn mock_handshake_function(
            channels: &Option<Vec<String>>,
            channel_groups: &Option<Vec<String>>,
            attempt: u8,
            reason: Option<PubNubError>,
        ) -> SubscribeEvent {
            assert_eq!(channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(attempt, 0);
            assert_eq!(reason, None);

            SubscribeEvent::HandshakeSuccess {
                cursor: SubscribeCursor {
                    timetoken: 0,
                    region: 0,
                },
            }
        }

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            mock_handshake_function,
        );

        assert!(result.is_some());
        assert!(!result.unwrap().is_empty())
    }
}
