use crate::{
    dx::subscribe::{
        event_engine::{effects::ReceiveEffectExecutor, SubscribeEvent},
        SubscribeCursor,
    },
    lib::alloc::{string::String, sync::Arc, vec::Vec},
};
use log::info;

pub(crate) fn execute(
    channels: &Option<Vec<String>>,
    channel_groups: &Option<Vec<String>>,
    cursor: &SubscribeCursor,
    _executor: &Arc<ReceiveEffectExecutor>,
) -> Option<Vec<SubscribeEvent>> {
    info!(
        "Receive at {:?} for\nchannels: {:?}\nchannel groups: {:?}",
        cursor.timetoken,
        channels.as_ref().unwrap_or(&Vec::new()),
        channel_groups.as_ref().unwrap_or(&Vec::new()),
    );

    // let result: Result<Vec<SubscribeEvent>, PubNubError> =
    //     executor(channels, channel_groups, cursor, 0, None);
    // Some(result.unwrap_or_else(|err| vec![SubscribeEvent::ReceiveFailure { reason: err }]))
    None
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::PubNubError, dx::subscribe::result::SubscribeResult};
    use futures::FutureExt;

    #[test]
    fn receive_messages() {
        let mock_receive_function: Arc<ReceiveEffectExecutor> =
            Arc::new(move |channels, channel_groups, cursor, attempt, reason| {
                assert_eq!(channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(attempt, 0);
                assert_eq!(reason, None);
                assert_eq!(cursor, &Default::default());

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
            &mock_receive_function,
        );

        assert!(matches!(
            result.unwrap().first().unwrap(),
            &SubscribeEvent::ReceiveSuccess { .. }
        ))
    }

    #[test]
    fn return_handshake_failure_event_on_err() {
        let mock_receive_function: Arc<ReceiveEffectExecutor> = Arc::new(move |_, _, _, _, _| {
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
            &mock_receive_function,
        )
        .unwrap();
        let result = &binding[0];

        assert!(matches!(result, &SubscribeEvent::ReceiveFailure { .. }));
    }
}
