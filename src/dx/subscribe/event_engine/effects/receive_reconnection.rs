use crate::{
    core::PubNubError,
    dx::subscribe::{
        event_engine::{effects::SubscribeEffectExecutor, SubscribeEvent},
        SubscribeCursor,
    },
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use log::info;

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    _attempt: u8,
    _reason: PubNubError,
    _executor: &Arc<SubscribeEffectExecutor>,
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
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[test]
    fn receive_messages() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |channels, channel_groups, cursor, attempt, reason| {
                assert_eq!(channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(attempt, 10);
                assert_eq!(
                    reason,
                    Some(PubNubError::Transport {
                        details: "test".into(),
                    })
                );
                assert_eq!(cursor, Some(&Default::default()));

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
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
            },
            &mock_receive_function,
        );

        assert!(matches!(
            result.unwrap().first().unwrap(),
            &SubscribeEvent::ReceiveSuccess { .. }
        ))
    }

    #[test]
    fn return_handshake_failure_event_on_err() {
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_, _, _, _, _| {
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
            &Default::default(),
            10,
            PubNubError::Transport {
                details: "test".into(),
            },
            &mock_receive_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(
            result,
            &SubscribeEvent::ReceiveReconnectFailure { .. }
        ));
    }
}
