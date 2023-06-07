use crate::dx::subscribe::{
    event_engine::{ReceiveFunction, SubscribeEvent},
    SubscribeCursor,
};
use crate::lib::alloc::{string::String, vec, vec::Vec};

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    executor: ReceiveFunction,
) -> Option<Vec<SubscribeEvent>> {
    Some(
        executor(channels, channel_groups, cursor, 0, None)
            .unwrap_or_else(|err| vec![SubscribeEvent::ReceiveFailure { reason: err }]),
    )
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::SubscribeCursor};

    #[test]
    fn receive_messages() {
        fn mock_receive_function(
            channels: &Option<Vec<String>>,
            channel_groups: &Option<Vec<String>>,
            cursor: &SubscribeCursor,
            attempt: u8,
            reason: Option<PubNubError>,
        ) -> Result<Vec<SubscribeEvent>, PubNubError> {
            assert_eq!(channels, &Some(vec!["ch1".to_string()]));
            assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
            assert_eq!(attempt, 0);
            assert_eq!(reason, None);
            assert_eq!(
                cursor,
                &SubscribeCursor {
                    timetoken: 0,
                    region: 0
                }
            );

            Ok(vec![SubscribeEvent::ReceiveSuccess {
                cursor: SubscribeCursor {
                    timetoken: 0,
                    region: 0,
                },
                messages: vec![],
            }])
        }

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            &SubscribeCursor {
                timetoken: 0,
                region: 0,
            },
            mock_receive_function,
        );

        assert!(matches!(
            result.unwrap().first().unwrap(),
            &SubscribeEvent::ReceiveSuccess { .. }
        ))
    }

    #[test]
    fn return_handskahe_failure_event_on_err() {
        fn mock_receive_function(
            _channels: &Option<Vec<String>>,
            _channel_groups: &Option<Vec<String>>,
            _cursor: &SubscribeCursor,
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
            &SubscribeCursor {
                timetoken: 0,
                region: 0,
            },
            mock_receive_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(result, &SubscribeEvent::ReceiveFailure { .. }));
    }
}
