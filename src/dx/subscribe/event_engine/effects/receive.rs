use crate::{
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
    _executor: &Arc<SubscribeEffectExecutor>,
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
        let mock_receive_function: Arc<SubscribeEffectExecutor> =
            Arc::new(move |cursor, params| {
                assert_eq!(params.channels, &Some(vec!["ch1".to_string()]));
                assert_eq!(params.channel_groups, &Some(vec!["cg1".to_string()]));
                assert_eq!(params._attempt, 0);
                assert_eq!(params._reason, None);
                assert_eq!(cursor, Some(&Default::default()));
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
        let mock_receive_function: Arc<SubscribeEffectExecutor> = Arc::new(move |_, _| {
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
