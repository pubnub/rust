use crate::{
    core::{
        event_engine::{State, Transition},
        PubNubError,
    },
    dx::subscribe::{
        event_engine::{
            SubscribeEffectInvocation::{
                self, CancelHandshake, CancelHandshakeReconnect, CancelReceive,
                CancelReceiveReconnect, EmitMessages, EmitStatus, Handshake, HandshakeReconnect,
                Receive, ReceiveReconnect,
            },
            SubscribeEvent,
        },
        result::Update,
        SubscribeCursor, SubscribeStatus,
    },
    lib::alloc::{string::String, vec, vec::Vec},
};

/// States of subscribe state machine.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) enum SubscribeState {
    /// Unsubscribed state.
    ///
    /// The initial state has no information about channels or groups from which
    /// events should be retrieved in real-time.
    Unsubscribed,

    /// Subscription initiation state.
    ///
    /// Retrieve the information that will be used to start the subscription
    /// loop.
    Handshaking {
        /// Optional list of channels.
        ///
        /// List of channels which will be source of real-time updates after
        /// initial subscription completion.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which will be source of real-time updates
        /// after initial subscription completion.
        channel_groups: Option<Vec<String>>,
    },

    /// Subscription recover state.
    ///
    /// The system is recovering after the initial subscription attempt failed.
    HandshakeReconnecting {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed initial
        /// subscription.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// initial subscription.
        channel_groups: Option<Vec<String>>,

        /// Current initial subscribe retry attempt.
        ///
        /// Used to track overall number of initial subscription retry attempts.
        attempts: u8,

        /// Initial subscribe attempt failure reason.
        reason: PubNubError,
    },

    /// Initial subscription stopped state.
    ///
    /// Subscription state machine state, which is set when
    /// [`SubscribeEvent::Disconnect`] event sent while in
    /// [`SubscribeState::Handshaking`] or
    /// [`SubscribeState::HandshakeReconnecting`] state.
    HandshakeStopped {
        /// Optional list of channels.
        ///
        /// List of channels for which initial subscription stopped.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which initial subscription stopped.
        channel_groups: Option<Vec<String>>,
    },

    /// Initial subscription failure state.
    ///
    /// System wasn't able to perform successful initial subscription after
    /// fixed number of attempts.
    HandshakeFailed {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed initial
        /// subscription.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// initial subscription.
        channel_groups: Option<Vec<String>>,

        /// Initial subscribe attempt failure reason.
        reason: PubNubError,
    },

    /// Receiving updates state.
    ///
    /// Subscription state machine is in state where it receive real-time
    /// updates from [`PubNub`] network.
    ///
    /// [`PubNub`]:https://www.pubnub.com/
    Receiving {
        /// Optional list of channels.
        ///
        /// List of channels for which real-time updates will be delivered.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which real-time updates will be
        /// delivered.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,
    },

    /// Subscription recover state.
    ///
    /// The system is recovering after the updates receiving attempt failed.
    ReceiveReconnecting {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed receive
        /// updates.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// receive updates.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,

        /// Current receive retry attempt.
        ///
        /// Used to track overall number of receive updates retry attempts.
        attempts: u8,

        /// Receive updates attempt failure reason.
        reason: PubNubError,
    },

    /// Updates receiving stopped state.
    ///
    /// Subscription state machine state, which is set when
    /// [`SubscribeEvent::Disconnect`] event sent while in
    /// [`SubscribeState::Handshaking`] or
    /// [`SubscribeState::HandshakeReconnecting`] state.
    ReceiveStopped {
        /// Optional list of channels.
        ///
        /// List of channels for which updates receive stopped.
        channels: Option<Vec<String>>,

        /// Optional list of channels.
        ///
        /// List of channel groups for which updates receive stopped.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,
    },

    /// Updates receiving failure state.
    ///
    /// System wasn't able to receive updates after fixed number of attempts.
    ReceiveFailed {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed receive
        /// updates.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// receive updates.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,

        /// Receive updates attempt failure reason.
        reason: PubNubError,
    },
}

impl SubscribeState {
    /// Handle channels / groups list change event.
    fn subscription_changed_transition(
        &self,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Unsubscribed
            | Self::Handshaking { .. }
            | Self::HandshakeReconnecting { .. }
            | Self::HandshakeFailed { .. } => Some(self.transition_to(
                Self::Handshaking {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                },
                None,
            )),
            Self::HandshakeStopped { .. } => Some(self.transition_to(
                Self::HandshakeStopped {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                },
                None,
            )),
            Self::Receiving { cursor, .. }
            | Self::ReceiveReconnecting { cursor, .. }
            | Self::ReceiveFailed { cursor, .. } => Some(self.transition_to(
                Self::Receiving {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                None,
            )),
            Self::ReceiveStopped { cursor, .. } => Some(self.transition_to(
                Self::ReceiveStopped {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                None,
            )),
        }
    }

    /// Handle catchup event.
    ///
    /// Event is sent each time during attempt to subscribe with specific
    /// `cursor`.
    fn subscription_restored_transition(
        &self,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
        cursor: &SubscribeCursor,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Unsubscribed
            | Self::Handshaking { .. }
            | Self::HandshakeReconnecting { .. }
            | Self::HandshakeFailed { .. }
            | Self::Receiving { .. }
            | Self::ReceiveReconnecting { .. }
            | Self::ReceiveFailed { .. } => Some(self.transition_to(
                Self::Receiving {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                None,
            )),
            Self::HandshakeStopped { .. } | Self::ReceiveStopped { .. } => {
                Some(self.transition_to(
                    Self::ReceiveStopped {
                        channels: channels.clone(),
                        channel_groups: channel_groups.clone(),
                        cursor: cursor.clone(),
                    },
                    None,
                ))
            }
        }
    }

    /// Handle initial (reconnect) handshake success event.
    ///
    /// Event is sent when provided set of channels and groups has been used for
    /// first time.
    fn handshake_success_transition(
        &self,
        cursor: &SubscribeCursor,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Handshaking {
                channels,
                channel_groups,
            }
            | Self::HandshakeReconnecting {
                channels,
                channel_groups,
                ..
            } => Some(self.transition_to(
                Self::Receiving {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                Some(vec![EmitStatus(SubscribeStatus::Connected)]),
            )),
            _ => None,
        }
    }

    /// Handle initial handshake failure event.
    fn handshake_failure_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Handshaking {
                channels,
                channel_groups,
            } => Some(self.transition_to(
                Self::HandshakeReconnecting {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    attempts: 1,
                    reason: reason.clone(),
                },
                None,
            )),
            _ => None,
        }
    }

    /// Handle handshake reconnect failure event.
    ///
    /// Event is sent if handshake reconnect effect failed due to any network
    /// issues.
    fn handshake_reconnect_failure_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::HandshakeReconnecting {
                channels,
                channel_groups,
                attempts,
                ..
            } => Some(self.transition_to(
                Self::HandshakeReconnecting {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    attempts: attempts + 1,
                    reason: reason.clone(),
                },
                None,
            )),
            _ => None,
        }
    }

    /// Handle handshake reconnection limit event.
    ///
    /// Event is sent if handshake reconnect reached maximum number of reconnect
    /// attempts.
    fn handshake_reconnect_give_up_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::HandshakeReconnecting {
                channels,
                channel_groups,
                ..
            } => Some(self.transition_to(
                Self::HandshakeFailed {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    reason: reason.clone(),
                },
                Some(vec![EmitStatus(SubscribeStatus::ConnectedError(
                    reason.clone(),
                ))]),
            )),
            _ => None,
        }
    }

    /// Handle updates receive (reconnect) success event.
    ///
    /// Event is sent when real-time updates received for previously subscribed
    /// channels / groups.
    fn receive_success_transition(
        &self,
        cursor: &SubscribeCursor,
        messages: &[Update],
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Receiving {
                channels,
                channel_groups,
                ..
            }
            | Self::ReceiveReconnecting {
                channels,
                channel_groups,
                ..
            } => Some(self.transition_to(
                Self::Receiving {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                Some(vec![
                    EmitMessages(messages.to_vec()),
                    EmitStatus(SubscribeStatus::Connected),
                ]),
            )),
            _ => None,
        }
    }

    /// Handle updates receive failure event.
    fn receive_failure_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Receiving {
                channels,
                channel_groups,
                cursor,
                ..
            } => Some(self.transition_to(
                Self::ReceiveReconnecting {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                    attempts: 1,
                    reason: reason.clone(),
                },
                None,
            )),
            _ => None,
        }
    }

    /// Handle updates receive failure event.
    ///
    /// Event is sent if updates receive effect failed due to any network
    /// issues.
    fn receive_reconnect_failure_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::ReceiveReconnecting {
                channels,
                channel_groups,
                attempts,
                cursor,
                ..
            } => Some(self.transition_to(
                Self::ReceiveReconnecting {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                    attempts: attempts + 1,
                    reason: reason.clone(),
                },
                None,
            )),
            _ => None,
        }
    }

    /// Handle receive updates reconnection limit event.
    ///
    /// Event is sent if receive updates reconnect reached maximum number of
    /// reconnect attempts.
    fn receive_reconnect_give_up_transition(
        &self,
        reason: &PubNubError,
    ) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::ReceiveReconnecting {
                channels,
                channel_groups,
                cursor,
                ..
            } => Some(self.transition_to(
                Self::ReceiveFailed {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                    reason: reason.clone(),
                },
                Some(vec![EmitStatus(SubscribeStatus::Disconnected)]),
            )),
            _ => None,
        }
    }

    /// Handle disconnect event.
    ///
    /// Event is sent each time when client asked to unsubscribe all
    /// channels / groups or temporally stop any activity.
    fn disconnect_transition(&self) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::Handshaking {
                channels,
                channel_groups,
            }
            | Self::HandshakeReconnecting {
                channels,
                channel_groups,
                ..
            } => Some(self.transition_to(
                Self::HandshakeStopped {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                },
                None,
            )),
            Self::Receiving {
                channels,
                channel_groups,
                cursor,
            }
            | Self::ReceiveReconnecting {
                channels,
                channel_groups,
                cursor,
                ..
            } => Some(self.transition_to(
                Self::ReceiveStopped {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                Some(vec![EmitStatus(SubscribeStatus::Disconnected)]),
            )),
            _ => None,
        }
    }

    /// Handle reconnect event.
    ///
    /// Event is sent each time when client asked to restore activity for
    /// channels / groups after which previously temporally stopped or restore
    /// after reconnection failures.
    fn reconnect_transition(&self) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        match self {
            Self::HandshakeStopped {
                channels,
                channel_groups,
                ..
            }
            | Self::HandshakeFailed {
                channels,
                channel_groups,
                ..
            } => Some(self.transition_to(
                Self::Handshaking {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                },
                None,
            )),
            Self::ReceiveStopped {
                channels,
                channel_groups,
                cursor,
            }
            | Self::ReceiveFailed {
                channels,
                channel_groups,
                cursor,
                ..
            } => Some(self.transition_to(
                Self::Receiving {
                    channels: channels.clone(),
                    channel_groups: channel_groups.clone(),
                    cursor: cursor.clone(),
                },
                None,
            )),
            _ => None,
        }
    }

    /// Handle unsubscribe all event.
    fn unsubscribe_all_transition(&self) -> Option<Transition<Self, SubscribeEffectInvocation>> {
        Some(self.transition_to(
            Self::Unsubscribed,
            Some(vec![EmitStatus(SubscribeStatus::Disconnected)]),
        ))
    }
}

impl State for SubscribeState {
    type State = Self;
    type Invocation = SubscribeEffectInvocation;
    type Event = SubscribeEvent;

    fn enter(&self) -> Option<Vec<Self::Invocation>> {
        match self {
            SubscribeState::Handshaking {
                channels,
                channel_groups,
            } => Some(vec![Handshake {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
            }]),
            Self::HandshakeReconnecting {
                channels,
                channel_groups,
                attempts,
                reason,
            } => Some(vec![HandshakeReconnect {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                attempts: *attempts,
                reason: reason.clone(),
            }]),
            Self::Receiving {
                channels,
                channel_groups,
                cursor,
            } => Some(vec![Receive {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: cursor.clone(),
            }]),
            Self::ReceiveReconnecting {
                channels,
                channel_groups,
                cursor,
                attempts,
                reason,
            } => Some(vec![ReceiveReconnect {
                channels: channels.clone(),
                channel_groups: channel_groups.clone(),
                cursor: cursor.clone(),
                attempts: *attempts,
                reason: reason.clone(),
            }]),
            _ => None,
        }
    }

    fn exit(&self) -> Option<Vec<Self::Invocation>> {
        match self {
            Self::Handshaking { .. } => Some(vec![CancelHandshake]),
            Self::HandshakeReconnecting { .. } => Some(vec![CancelHandshakeReconnect]),
            Self::Receiving { .. } => Some(vec![CancelReceive]),
            Self::ReceiveReconnecting { .. } => Some(vec![CancelReceiveReconnect]),
            _ => None,
        }
    }

    fn transition(&self, event: &Self::Event) -> Option<Transition<Self::State, Self::Invocation>> {
        match event {
            SubscribeEvent::SubscriptionChanged {
                channels,
                channel_groups,
            } => self.subscription_changed_transition(channels, channel_groups),
            SubscribeEvent::SubscriptionRestored {
                channels,
                channel_groups,
                cursor,
            } => self.subscription_restored_transition(channels, channel_groups, cursor),
            SubscribeEvent::HandshakeSuccess { cursor }
            | SubscribeEvent::HandshakeReconnectSuccess { cursor } => {
                self.handshake_success_transition(cursor)
            }
            SubscribeEvent::HandshakeFailure { reason } => {
                self.handshake_failure_transition(reason)
            }
            SubscribeEvent::HandshakeReconnectFailure { reason, .. } => {
                self.handshake_reconnect_failure_transition(reason)
            }
            SubscribeEvent::HandshakeReconnectGiveUp { reason } => {
                self.handshake_reconnect_give_up_transition(reason)
            }
            SubscribeEvent::ReceiveSuccess { cursor, messages }
            | SubscribeEvent::ReceiveReconnectSuccess { cursor, messages } => {
                self.receive_success_transition(cursor, messages)
            }
            SubscribeEvent::ReceiveFailure { reason } => self.receive_failure_transition(reason),
            SubscribeEvent::ReceiveReconnectFailure { reason } => {
                self.receive_reconnect_failure_transition(reason)
            }
            SubscribeEvent::ReceiveReconnectGiveUp { reason } => {
                self.receive_reconnect_give_up_transition(reason)
            }
            SubscribeEvent::Disconnect => self.disconnect_transition(),
            SubscribeEvent::Reconnect => self.reconnect_transition(),
            SubscribeEvent::UnsubscribeAll => self.unsubscribe_all_transition(),
        }
    }

    fn transition_to(
        &self,
        state: Self::State,
        invocations: Option<Vec<Self::Invocation>>,
    ) -> Transition<Self::State, Self::Invocation> {
        Transition {
            invocations: self
                .exit()
                .unwrap_or(vec![])
                .into_iter()
                .chain(invocations.unwrap_or(vec![]).into_iter())
                .chain(state.enter().unwrap_or(vec![]).into_iter())
                .collect(),
            state,
        }
    }
}

#[cfg(test)]
mod should {
    // TODO: EE process tests should be async!

    use super::*;
    use crate::core::RequestRetryPolicy;
    use crate::providers::futures_tokio::TokioRuntime;
    use crate::{
        core::event_engine::EventEngine,
        dx::subscribe::{
            event_engine::{
                effects::{
                    EmitMessagesEffectExecutor, EmitStatusEffectExecutor, SubscribeEffectExecutor,
                },
                SubscribeEffect, SubscribeEffectHandler,
            },
            result::SubscribeResult,
        },
        lib::alloc::sync::Arc,
    };
    use futures::FutureExt;
    use test_case::test_case;

    fn event_engine(
        start_state: SubscribeState,
    ) -> Arc<
        EventEngine<
            SubscribeState,
            SubscribeEffectHandler,
            SubscribeEffect,
            SubscribeEffectInvocation,
        >,
    > {
        let call: Arc<SubscribeEffectExecutor> = Arc::new(|_| {
            async move {
                Ok(SubscribeResult {
                    cursor: Default::default(),
                    messages: vec![],
                })
            }
            .boxed()
        });

        let emit_status: Arc<EmitStatusEffectExecutor> = Arc::new(|_| {});
        let emit_message: Arc<EmitMessagesEffectExecutor> = Arc::new(|_| {});

        let (tx, _) = async_channel::bounded(1);

        EventEngine::new(
            SubscribeEffectHandler::new(
                call,
                emit_status,
                emit_message,
                RequestRetryPolicy::None,
                tx,
            ),
            start_state,
            TokioRuntime,
        )
    }

    #[test_case(
        SubscribeState::Unsubscribed,
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        };
        "to handshaking on subscription changed"
    )]
    #[test_case(
        SubscribeState::Unsubscribed,
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to handshaking on subscription restored"
    )]
    #[test_case(
        SubscribeState::Unsubscribed,
        SubscribeEvent::ReceiveFailure {
            reason: PubNubError::Transport { details: "Test".to_string(), response: None }
        },
        SubscribeState::Unsubscribed;
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_for_unsubscribed_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        // Process event.
        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Handshaking {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()])
        };
        "to handshaking on subscription changed"
    )]
    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::HandshakeFailure {
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts:  1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to handshake reconnect on handshake failure"
    )]
    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::Disconnect,
        SubscribeState::HandshakeStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        };
        "to handshake stopped on disconnect"
    )]
    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::HandshakeSuccess {
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to receiving on handshake success"
    )]
    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        },
        SubscribeEvent::HandshakeReconnectGiveUp {
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, }
        },
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()])
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_handshaking_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::HandshakeReconnectFailure {
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        },
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 2,
            reason: PubNubError::Transport { details: "Test reason on error".to_string(), response: None, },
        };
        "to handshake reconnecting on reconnect failure"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Handshaking {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        };
        "to handshaking on subscription change"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::Disconnect,
        SubscribeState::HandshakeStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        };
        "to handshake stopped on disconnect"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::HandshakeReconnectGiveUp {
            reason: PubNubError::Transport { details: "Test give up reason".to_string(), response: None, }
        },
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test give up reason".to_string(), response: None, }
        };
        "to handshake failed on give up"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::HandshakeReconnectSuccess {
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to receiving on reconnect success"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::ReceiveSuccess {
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            messages: vec![]
        },
        SubscribeState::HandshakeReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_handshake_reconnecting_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Handshaking {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        };
        "to handshaking on subscription changed"
    )]
    #[test_case(
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::Reconnect,
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        };
        "to handshaking on reconnect"
    )]
    #[test_case(
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 }
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        },
        SubscribeEvent::ReceiveSuccess {
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            messages: vec![]
        },
        SubscribeState::HandshakeFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_handshake_failed_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::HandshakeStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        SubscribeEvent::Reconnect,
        SubscribeState::Handshaking {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        };
        "to handshaking on reconnect"
    )]
    #[test_case(
        SubscribeState::HandshakeStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        },
        SubscribeEvent::ReceiveSuccess {
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            messages: vec![]
        },
        SubscribeState::HandshakeStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_handshake_stopped_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receiving on subscription changed"
    )]
    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::ReceiveSuccess {
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
            messages: vec![]
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        };
        "to receiving on receive success"
    )]
    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::ReceiveFailure {
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, }
        },
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test reason".to_string(), response: None, }
        };
        "to receive reconnecting on receive failure"
    )]
    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::Disconnect,
        SubscribeState::ReceiveStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receive stopped on disconnect"
    )]
    #[test_case(
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::HandshakeSuccess {
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_receiving_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::ReceiveReconnectFailure {
            reason: PubNubError::Transport { details: "Test reconnect error".to_string(), response: None, }
        },
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 2,
            reason: PubNubError::Transport { details: "Test reconnect error".to_string(), response: None, }
        };
        "to receive reconnecting on reconnect failure"
    )]
    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receiving on subscription changed"
    )]
    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::Disconnect,
        SubscribeState::ReceiveStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receive stopped on disconnect"
    )]
    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::ReceiveReconnectGiveUp {
            reason: PubNubError::Transport { details: "Test give up error".to_string(), response: None, }
        },
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test give up error".to_string(), response: None, }
        };
        "to receive failed on give up"
    )]
    #[test_case(
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::HandshakeSuccess {
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        },
        SubscribeState::ReceiveReconnecting {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            attempts: 1,
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_receiving_reconnecting_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }

    #[test_case(
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::SubscriptionChanged {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receiving on subscription changed"
    )]
    #[test_case(
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::SubscriptionRestored {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        },
        SubscribeState::Receiving {
            channels: Some(vec!["ch2".to_string()]),
            channel_groups: Some(vec!["gr2".to_string()]),
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 },
        };
        "to receiving on subscription restored"
    )]
    #[test_case(
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::Reconnect,
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receiving on reconnect"
    )]
    #[test_case(
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        },
        SubscribeEvent::HandshakeSuccess {
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 }
        },
        SubscribeState::ReceiveFailed {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
            reason: PubNubError::Transport { details: "Test error".to_string(), response: None, }
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_receive_failed_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }
    #[test_case(
        SubscribeState::ReceiveStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::Reconnect,
        SubscribeState::Receiving {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to receiving on reconnect"
    )]
    #[test_case(
        SubscribeState::ReceiveStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        },
        SubscribeEvent::HandshakeSuccess {
            cursor: SubscribeCursor { timetoken: "100".into(), region: 1 }
        },
        SubscribeState::ReceiveStopped {
            channels: Some(vec!["ch1".to_string()]),
            channel_groups: Some(vec!["gr1".to_string()]),
            cursor: SubscribeCursor { timetoken: "10".into(), region: 1 },
        };
        "to not change on unexpected event"
    )]
    #[tokio::test]
    async fn transition_receive_stopped_state(
        init_state: SubscribeState,
        event: SubscribeEvent,
        target_state: SubscribeState,
    ) {
        let engine = event_engine(init_state.clone());
        assert_eq!(engine.current_state(), init_state);

        engine.process(&event);

        assert_eq!(engine.current_state(), target_state);
    }
}
