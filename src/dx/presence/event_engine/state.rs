//! # Heartbeat event engine state module.
//!
//! The module contains the [`PresenceState`] type, which describes available
//! event engine states. The module also contains an implementation of
//! `transition` between states in response to certain events.

use crate::{
    core::{
        event_engine::{EffectInvocation, State, Transition},
        PubNubError,
    },
    lib::alloc::string::String,
    presence::event_engine::{
        PresenceEffectInvocation::{self, *},
        PresenceEvent, PresenceInput,
    },
};

/// Available `Heartbeat` event engine states.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PresenceState {
    /// Inactive heartbeat state.
    ///
    /// The initial state jas no information about channels or groups for which
    /// event engine should keep presence of `user_id`.
    Inactive,

    /// Heartbeating state.
    ///
    /// Sending explicit heartbeat request for known channels and groups to the
    /// [`PubNub`] network.
    ///
    /// [`PubNub`]:https://www.pubnub.com/
    Heartbeating {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced.
        input: PresenceInput,
    },

    /// Cooling down state.
    ///
    /// Heartbeating idle state in which it stays for configured amount of time.
    Cooldown {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence should be announced after configured heartbeat interval.
        input: PresenceInput,
    },

    /// Heartbeat recovering state.
    ///
    /// The system is recovering after heartbeating attempt failure.
    Reconnecting {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence will be announced after heartbeating restore.
        input: PresenceInput,

        /// Current heartbeating retry attempt.
        ///
        /// Used to track overall number of heartbeating retry attempts.
        attempts: u8,

        /// Heartbeating attempt failure reason.
        reason: PubNubError,
    },

    /// Heartbeat stopped state.
    ///
    /// Heartbeat explicitly has been stopped in response on user actions with
    /// subscription.
    Stopped {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence will be announced after heartbeating restore.
        input: PresenceInput,
    },

    /// Heartbeating failure state.
    ///
    /// System wasn't able to perform heartbeating after fixed number of
    /// attempts.
    Failed {
        /// User input with channels and groups.
        ///
        /// Object contains list of channels and groups for which `user_id`
        /// presence will be announced after heartbeating restore.
        input: PresenceInput,

        /// Heartbeating attempt failure reason.
        reason: PubNubError,
    },
}

impl PresenceState {
    /// Handle `joined` event.
    fn presence_joined_transition(
        &self,
        heartbeat_interval: u64,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        if heartbeat_interval == 0 {
            return None;
        }

        let event_input = PresenceInput::new(channels, channel_groups);

        match self {
            Self::Inactive => {
                Some(self.transition_to(Some(Self::Heartbeating { input: event_input }), None))
            }
            Self::Heartbeating { input }
            | Self::Cooldown { input }
            | Self::Reconnecting { input, .. }
            | Self::Failed { input, .. }
            | Self::Stopped { input }
                if &event_input != input =>
            {
                let input = input.clone() + event_input;
                Some(self.transition_to(
                    Some(if !matches!(self, Self::Stopped { .. }) {
                        Self::Heartbeating { input }
                    } else {
                        Self::Stopped { input }
                    }),
                    None,
                ))
            }
            _ => None,
        }
    }

    /// Handle `left` event.
    fn presence_left_transition(
        &self,
        suppress_leave_events: bool,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        let event_input = PresenceInput::new(channels, channel_groups);

        match self {
            Self::Heartbeating { input }
            | Self::Cooldown { input }
            | Self::Reconnecting { input, .. }
            | Self::Failed { input, .. }
            | Self::Stopped { input } => {
                let channels_for_heartbeating = input.clone() - event_input.clone();
                // Calculate actual list for which `Leave` invocation should be created.
                let channels_to_leave = input.clone() - channels_for_heartbeating.clone();

                (!channels_to_leave.is_empty).then(|| {
                    self.transition_to(
                        Some(if !channels_for_heartbeating.is_empty {
                            if !matches!(self, Self::Stopped { .. }) {
                                Self::Heartbeating {
                                    input: channels_for_heartbeating,
                                }
                            } else {
                                Self::Stopped {
                                    input: channels_for_heartbeating,
                                }
                            }
                        } else {
                            Self::Inactive
                        }),
                        (!matches!(self, Self::Stopped { .. }) && !suppress_leave_events).then(
                            || {
                                vec![Leave {
                                    input: channels_to_leave,
                                }]
                            },
                        ),
                    )
                })
            }
            _ => None,
        }
    }

    /// Handle `left all` event.
    fn presence_left_all_transition(
        &self,
        suppress_leave_events: bool,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Heartbeating { input }
            | Self::Cooldown { input }
            | Self::Reconnecting { input, .. }
            | Self::Failed { input, .. } => Some(self.transition_to(
                Some(Self::Inactive),
                (!suppress_leave_events).then(|| {
                    vec![Leave {
                        input: input.clone(),
                    }]
                }),
            )),
            Self::Stopped { .. } => Some(self.transition_to(Some(Self::Inactive), None)),
            _ => None,
        }
    }

    /// Handle `heartbeat success` event.
    fn presence_heartbeat_success_transition(
        &self,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Heartbeating { input } | Self::Reconnecting { input, .. } => {
                Some(self.transition_to(
                    Some(Self::Cooldown {
                        input: input.clone(),
                    }),
                    None,
                ))
            }
            _ => None,
        }
    }

    /// Handle `heartbeat failure` event.
    fn presence_heartbeat_failed_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        // Request cancellation shouldn't cause any transition because there
        // will be another event after this.
        if matches!(reason, PubNubError::RequestCancel { .. }) {
            return None;
        }

        match self {
            Self::Heartbeating { input } => Some(self.transition_to(
                Some(Self::Reconnecting {
                    input: input.clone(),
                    attempts: 1,
                    reason: reason.clone(),
                }),
                None,
            )),
            Self::Reconnecting {
                input, attempts, ..
            } => Some(self.transition_to(
                Some(Self::Reconnecting {
                    input: input.clone(),
                    attempts: attempts + 1,
                    reason: reason.clone(),
                }),
                None,
            )),
            _ => None,
        }
    }

    /// Handle `heartbeat give up` event.
    fn presence_heartbeat_give_up_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Reconnecting { input, .. } => Some(self.transition_to(
                Some(Self::Failed {
                    input: input.clone(),
                    reason: reason.clone(),
                }),
                None,
            )),
            _ => None,
        }
    }

    /// Handle `reconnect` event.
    fn presence_reconnect_transition(&self) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Stopped { input } | Self::Failed { input, .. } => Some(self.transition_to(
                Some(Self::Heartbeating {
                    input: input.clone(),
                }),
                None,
            )),
            _ => None,
        }
    }

    /// Handle `reconnect` event.
    fn presence_disconnect_transition(&self) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Heartbeating { input }
            | Self::Cooldown { input }
            | Self::Reconnecting { input, .. }
            | Self::Failed { input, .. } => Some(self.transition_to(
                Some(Self::Stopped {
                    input: input.clone(),
                }),
                Some(vec![Leave {
                    input: input.clone(),
                }]),
            )),
            _ => None,
        }
    }

    /// Handle cooldown `times up` event.
    fn presence_times_up_transition(&self) -> Option<Transition<Self, PresenceEffectInvocation>> {
        match self {
            Self::Cooldown { input } => Some(self.transition_to(
                Some(Self::Heartbeating {
                    input: input.clone(),
                }),
                None,
            )),
            _ => None,
        }
    }
}

impl State for PresenceState {
    type State = Self;
    type Invocation = PresenceEffectInvocation;
    type Event = PresenceEvent;

    fn enter(&self) -> Option<Vec<Self::Invocation>> {
        match self {
            Self::Heartbeating { input } => Some(vec![Heartbeat {
                input: input.clone(),
            }]),
            Self::Cooldown { input } => Some(vec![Wait {
                input: input.clone(),
            }]),
            Self::Reconnecting {
                input,
                attempts,
                reason,
            } => Some(vec![DelayedHeartbeat {
                input: input.clone(),
                attempts: *attempts,
                reason: reason.clone(),
            }]),
            _ => None,
        }
    }

    fn exit(&self) -> Option<Vec<Self::Invocation>> {
        log::debug!("~~~~~~~~~~ EXIT: {self:?}");
        match self {
            PresenceState::Cooldown { .. } => Some(vec![CancelWait]),
            PresenceState::Reconnecting { .. } => Some(vec![CancelDelayedHeartbeat]),
            _ => None,
        }
    }

    fn transition(
        &self,
        event: &<<Self as State>::Invocation as EffectInvocation>::Event,
    ) -> Option<Transition<Self::State, Self::Invocation>> {
        match event {
            PresenceEvent::Joined {
                heartbeat_interval,
                channels,
                channel_groups,
            } => self.presence_joined_transition(*heartbeat_interval, channels, channel_groups),
            PresenceEvent::Left {
                suppress_leave_events,
                channels,
                channel_groups,
            } => self.presence_left_transition(*suppress_leave_events, channels, channel_groups),
            PresenceEvent::LeftAll {
                suppress_leave_events,
            } => self.presence_left_all_transition(*suppress_leave_events),
            PresenceEvent::HeartbeatSuccess => self.presence_heartbeat_success_transition(),
            PresenceEvent::HeartbeatFailure { reason } => {
                self.presence_heartbeat_failed_transition(reason)
            }
            PresenceEvent::HeartbeatGiveUp { reason } => {
                self.presence_heartbeat_give_up_transition(reason)
            }
            PresenceEvent::Reconnect => self.presence_reconnect_transition(),
            PresenceEvent::Disconnect => self.presence_disconnect_transition(),
            PresenceEvent::TimesUp => self.presence_times_up_transition(),
        }
    }

    fn transition_to(
        &self,
        state: Option<Self::State>,
        invocations: Option<Vec<Self::Invocation>>,
    ) -> Transition<Self::State, Self::Invocation> {
        let on_enter_invocations = match state.clone() {
            Some(state) => state.enter().unwrap_or_default(),
            None => vec![],
        };

        let invocations = self
            .exit()
            .unwrap_or_default()
            .into_iter()
            .chain(invocations.unwrap_or_default())
            .chain(on_enter_invocations)
            .collect();

        log::debug!("~~~~~~>> COLLECTED INVOCATIONS: {invocations:?}");

        Transition { invocations, state }
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::presence::event_engine::effects::LeaveEffectExecutor;
    use crate::presence::LeaveResult;
    use crate::{
        core::{event_engine::EventEngine, RequestRetryConfiguration},
        lib::alloc::sync::Arc,
        presence::{
            event_engine::{
                effects::{HeartbeatEffectExecutor, WaitEffectExecutor},
                PresenceEffectHandler, PresenceEventEngine,
            },
            HeartbeatResult,
        },
        providers::futures_tokio::RuntimeTokio,
    };
    use futures::FutureExt;
    use test_case::test_case;

    fn event_engine(start_state: PresenceState) -> Arc<PresenceEventEngine> {
        let heartbeat_call: Arc<HeartbeatEffectExecutor> =
            Arc::new(|_| async move { Ok(HeartbeatResult) }.boxed());
        let delayed_heartbeat_call: Arc<HeartbeatEffectExecutor> =
            Arc::new(|_| async move { Ok(HeartbeatResult) }.boxed());
        let leave_call: Arc<LeaveEffectExecutor> =
            Arc::new(|_| async move { Ok(LeaveResult) }.boxed());
        let wait_call: Arc<WaitEffectExecutor> = Arc::new(|_| async move { Ok(()) }.boxed());

        let (tx, _) = async_channel::bounded(1);

        EventEngine::new(
            PresenceEffectHandler::new(
                heartbeat_call,
                delayed_heartbeat_call,
                leave_call,
                wait_call,
                RequestRetryConfiguration::None,
                tx,
            ),
            start_state,
            RuntimeTokio,
        )
    }

    #[test_case(
        PresenceState::Inactive,
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Inactive,
        PresenceEvent::HeartbeatFailure {
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        },
        PresenceState::Inactive;
        "to not change on unexpected event"
    )]
    #[test_case(
        PresenceState::Inactive,
        PresenceEvent::Joined {
            heartbeat_interval: 0,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Inactive;
        "to not change with 0 presence interval"
    )]
    #[tokio::test]
    async fn transition_for_inactive_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Inactive));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr2".to_string()])
            )
        };
        "to heartbeating on left"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string(), "ch2".to_string()]),
            channel_groups: Some(vec!["gr1".to_string(), "gr2".to_string()]),
        },
        PresenceState::Inactive;
        "to inactive on left for all channels and groups"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::HeartbeatSuccess,
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to heartbeat cool down on heartbeat success"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::HeartbeatFailure {
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        },
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        };
        "to reconnect on heartbeat failure"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Disconnect,
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to stopped on disconnect"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::LeftAll {
            suppress_leave_events: false
        },
        PresenceState::Inactive;
        "to inactive on left all"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on joined with same channels and groups"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr3".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to not change on left with unknown channels and groups"
    )]
    #[test_case(
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::HeartbeatGiveUp {
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_heartbeating_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Heartbeating { .. }));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: None,
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on left"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string(), "ch2".to_string()]),
            channel_groups: Some(vec!["gr1".to_string(), "gr2".to_string()]),
        },
        PresenceState::Inactive;
        "to inactive on left for all channels and groups"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::TimesUp,
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to heartbeating on times up"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Disconnect,
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to stopped on disconnect"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::LeftAll {
            suppress_leave_events: false,
        },
        PresenceState::Inactive;
        "to inactive on left all"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on joined with same channels and groups"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr3".to_string()]),
        },
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on left with unknown channels and groups"
    )]
    #[test_case(
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::HeartbeatGiveUp {
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        },
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_cool_down_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Cooldown { .. }));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::HeartbeatFailure {
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        },
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 2,
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        };
        "to heartbeat reconnecting on heartbeat failure"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: None,
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on left"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string(), "ch2".to_string()]),
            channel_groups: Some(vec!["gr1".to_string(), "gr2".to_string()]),
        },
        PresenceState::Inactive;
        "to inactive on left for all channels and groups"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::HeartbeatSuccess,
        PresenceState::Cooldown {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to cool down on heartbeat success"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::HeartbeatGiveUp {
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        },
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        };
        "to failed on heartbeat give up"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Disconnect,
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to stopped on disconnect"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::LeftAll {
            suppress_leave_events: false,
        },
        PresenceState::Inactive;
        "to inactive on left all"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on joined with same channels and groups"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr3".to_string()]),
        },
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on left with unknown channels and groups"
    )]
    #[test_case(
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Reconnect,
        PresenceState::Reconnecting {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_reconnecting_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Reconnecting { .. }));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: None,
        },
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on left"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string(), "ch2".to_string()]),
            channel_groups: Some(vec!["gr1".to_string(), "gr2".to_string()]),
        },
        PresenceState::Inactive;
        "to inactive on left for all channels and groups"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Reconnect,
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to heartbeating on reconnect"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::LeftAll {
            suppress_leave_events: false,
        },
        PresenceState::Inactive;
        "to inactive on left all"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on joined with same channels and groups"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr3".to_string()]),
        },
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on left with unknown channels and groups"
    )]
    #[test_case(
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        },
        PresenceEvent::Disconnect,
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_stopped_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Stopped { .. }));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on joined"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: None,
        },
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            )
        };
        "to heartbeating on left"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string(), "ch2".to_string()]), 
                &Some(vec!["gr1".to_string(), "gr2".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: Some(vec!["ch1".to_string(), "ch2".to_string()]),
            channel_groups: Some(vec!["gr1".to_string(), "gr2".to_string()]),
        },
        PresenceState::Inactive;
        "to inactive on left for all channels and groups"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Reconnect,
        PresenceState::Heartbeating {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to heartbeating on reconnect"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Disconnect,
        PresenceState::Stopped {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            )
        };
        "to stopped on disconnect"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::LeftAll {
            suppress_leave_events: false,
        },
        PresenceState::Inactive;
        "to inactive on left all"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Joined {
            heartbeat_interval: 10,
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on joined with same channels and groups"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::Left {
            suppress_leave_events: false,
            channels: None,
            channel_groups: Some(vec!["gr3".to_string()]),
        },
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on left with unknown channels and groups"
    )]
    #[test_case(
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        PresenceEvent::HeartbeatSuccess,
        PresenceState::Failed {
            input: PresenceInput::new(
                &Some(vec!["ch1".to_string()]), 
                &Some(vec!["gr1".to_string()])
            ),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_failed_state(
        init_state: PresenceState,
        event: PresenceEvent,
        target_state: PresenceState,
    ) {
        let engine = event_engine(init_state.clone());
        assert!(matches!(init_state, PresenceState::Failed { .. }));
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }
}
