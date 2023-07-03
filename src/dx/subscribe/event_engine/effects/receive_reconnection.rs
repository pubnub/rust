use crate::dx::subscribe::event_engine::effects::ReceiveEffectExecutor;
use crate::lib::alloc::{string::String, vec, vec::Vec};
use crate::{
    core::PubNubError,
    dx::subscribe::{event_engine::SubscribeEvent, SubscribeCursor},
};
use log::info;
use std::sync::Arc;

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    attempt: u8,
    reason: PubNubError,
    executor: &Arc<Box<ReceiveEffectExecutor>>,
) -> Option<Vec<SubscribeEvent>> {
    info!(
        "Receive reconnection at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    // let result: Result<Vec<SubscribeEvent>, PubNubError> =
    //     executor(channels, channel_groups, cursor, attempt, Some(reason));
    // Some(result.unwrap_or_else(|err| vec![SubscribeEvent::ReceiveReconnectFailure { reason: err }]))
    None
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
            assert_eq!(attempt, 10);
            assert_eq!(
                reason,
                Some(PubNubError::Transport {
                    details: "test".into(),
                })
            );
            assert_eq!(cursor, &Default::default());

            Ok(vec![SubscribeEvent::ReceiveSuccess {
                cursor: Default::default(),
                messages: vec![],
            }])
        }

        let result = execute(
            &Some(vec!["ch1".to_string()]),
            &Some(vec!["cg1".to_string()]),
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
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
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
            },
            mock_receive_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(
            result,
            &SubscribeEvent::ReceiveReconnectFailure { .. }
        ));
    }
}
