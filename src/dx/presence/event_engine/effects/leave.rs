//! Presence leave module.
//!
//! Module contains implementation of `Leave` effect which is used to announce
//! `user_id` `leave` from specified channels and groups.

use log::info;

use crate::{
    lib::alloc::{sync::Arc, vec, vec::Vec},
    presence::event_engine::{
        effects::LeaveEffectExecutor,
        types::{PresenceInput, PresenceParameters},
        PresenceEvent,
    },
};

#[allow(clippy::too_many_arguments, dead_code)]
pub(super) async fn execute(
    input: &PresenceInput,
    effect_id: &str,
    executor: &Arc<LeaveEffectExecutor>,
) -> Vec<PresenceEvent> {
    let channel_groups = input.channel_groups();
    let channels = input.channels();

    info!(
        "Leave\nchannels: {:?}\nchannel groups: {:?}",
        channels, channel_groups
    );

    let _ = executor(PresenceParameters {
        channels: &channels,
        channel_groups: &channel_groups,
        attempt: 0,
        reason: None,
        effect_id,
    })
    .await;

    vec![]
}

#[cfg(test)]
mod it_should {
    use futures::FutureExt;

    use super::*;
    use crate::{
        core::{PubNubError, TransportResponse},
        presence::LeaveResult,
    };

    #[tokio::test]
    async fn return_leave_success_event() {
        let mocked_leave_function: Arc<LeaveEffectExecutor> = Arc::new(move |parameters| {
            assert_eq!(parameters.channel_groups, &Some(vec!["cg2".to_string()]));
            assert_eq!(parameters.channels, &Some(vec!["ch2".to_string()]));
            assert_eq!(parameters.attempt, 0);
            assert_eq!(
                parameters.reason.unwrap(),
                PubNubError::Transport {
                    details: "test".into(),
                    response: None
                }
            );
            assert_eq!(parameters.effect_id, "id");

            async move { Ok(LeaveResult) }.boxed()
        });

        let result = execute(
            &PresenceInput::new(
                &Some(vec!["ch2".to_string()]),
                &Some(vec!["cg2".to_string()]),
            ),
            "id",
            &mocked_leave_function,
        )
        .await;

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn return_heartbeat_failed_event_on_error() {
        let mocked_leave_function: Arc<LeaveEffectExecutor> = Arc::new(move |_| {
            async move {
                Err(PubNubError::Transport {
                    details: "test".into(),
                    response: Some(Box::new(TransportResponse {
                        status: 500,
                        ..Default::default()
                    })),
                })
            }
            .boxed()
        });

        let result = execute(
            &PresenceInput::new(
                &Some(vec!["ch3".to_string()]),
                &Some(vec!["cg3".to_string()]),
            ),
            "id",
            &mocked_leave_function,
        )
        .await;

        assert!(!result.is_empty());
        assert!(matches!(
            result.first().unwrap(),
            PresenceEvent::HeartbeatFailure { .. }
        ));
    }
}
