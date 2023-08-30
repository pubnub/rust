//! Subscriptions' manager.
//!
//! This module contains manager which is responsible for tracking and updating
//! active subscription streams.

use crate::{
    dx::subscribe::{
        event_engine::{event::SubscribeEvent, SubscribeEventEngine, SubscribeInput},
        result::Update,
        subscription::Subscription,
        SubscribeCursor, SubscribeStatus,
    },
    lib::{
        alloc::{sync::Arc, vec::Vec},
        core::{
            fmt::Debug,
            ops::{Deref, DerefMut},
        },
    },
};
use std::fmt::Formatter;

pub(in crate::dx::subscribe) type PresenceCall =
    dyn Fn(Option<Vec<String>>, Option<Vec<String>>) + Send + Sync;

/// Active subscriptions manager.
///
/// [`PubNubClient`] allows to have multiple [`subscription`] objects which will
/// be used to deliver real-time updates on channels and groups specified during
/// [`subscribe`] method call.
///
/// [`subscription`]: crate::Subscription
/// [`PubNubClient`]: crate::PubNubClient
#[derive(Debug)]
pub(crate) struct SubscriptionManager {
    pub(crate) inner: Arc<SubscriptionManagerRef>,
}

impl SubscriptionManager {
    pub fn new(
        event_engine: Arc<SubscribeEventEngine>,
        heartbeat_call: Arc<PresenceCall>,
        leave_call: Arc<PresenceCall>,
    ) -> Self {
        Self {
            inner: Arc::new(SubscriptionManagerRef {
                event_engine,
                subscribers: Default::default(),
                _heartbeat_call: heartbeat_call,
                _leave_call: leave_call,
            }),
        }
    }
}

impl Deref for SubscriptionManager {
    type Target = SubscriptionManagerRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubscriptionManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Presence configuration is not unique.")
    }
}

impl Clone for SubscriptionManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Active subscriptions manager.
///
/// [`PubNubClient`] allows to have multiple [`subscription`] objects which will
/// be used to deliver real-time updates on channels and groups specified during
/// [`subscribe`] method call.
///
/// [`subscription`]: crate::Subscription
/// [`PubNubClient`]: crate::PubNubClient
pub(crate) struct SubscriptionManagerRef {
    /// Subscription event engine.
    ///
    /// State machine which is responsible for subscription loop maintenance.
    event_engine: Arc<SubscribeEventEngine>,

    /// List of registered subscribers.
    ///
    /// List of subscribers which will receive real-time updates.
    subscribers: Vec<Subscription>,

    /// Presence `join` announcement.
    ///
    /// Announces `user_id` presence on specified channels and groups.
    _heartbeat_call: Arc<PresenceCall>,

    /// Presence `leave` announcement.
    ///
    /// Announces `user_id` `leave` from specified channels and groups.
    _leave_call: Arc<PresenceCall>,
}

impl SubscriptionManagerRef {
    pub fn notify_new_status(&self, status: &SubscribeStatus) {
        self.subscribers.iter().for_each(|subscription| {
            subscription.handle_status(status.clone());
        });
    }

    pub fn notify_new_messages(&self, messages: Vec<Update>) {
        self.subscribers.iter().for_each(|subscription| {
            subscription.handle_messages(&messages);
        });
    }

    pub fn register(&mut self, subscription: Subscription) {
        let cursor = subscription.cursor;
        self.subscribers.push(subscription);

        if let Some(cursor) = cursor {
            self.restore_subscription(cursor);
        } else {
            self.change_subscription(None);
        }
    }

    pub fn unregister(&mut self, subscription: Subscription) {
        if let Some(position) = self
            .subscribers
            .iter()
            .position(|val| val.id.eq(&subscription.id))
        {
            self.subscribers.swap_remove(position);
        }

        self.change_subscription(Some(&subscription.input));
    }

    pub fn unregister_all(&mut self) {
        let inputs = self.current_input();

        self.subscribers
            .iter_mut()
            .for_each(|subscription| subscription.invalidate());
        self.subscribers.clear();
        self.change_subscription(Some(&inputs));
    }

    pub fn disconnect(&self) {
        self.event_engine.process(&SubscribeEvent::Disconnect);
    }

    pub fn reconnect(&self) {
        self.event_engine.process(&SubscribeEvent::Reconnect);
    }

    fn change_subscription(&self, _removed: Option<&SubscribeInput>) {
        let inputs = self.current_input();

        // TODO: Uncomment after contract test server fix.
        // #[cfg(feature = "presence")]
        // {
        //     (!inputs.is_empty)
        //         .then(|| self.heartbeat_call.as_ref()(inputs.channels(),
        // inputs.channel_groups()));
        //
        //     if let Some(removed) = removed {
        //         (!removed.is_empty).then(|| {
        //             self.leave_call.as_ref()(removed.channels(),
        // removed.channel_groups())         });
        //     }
        // }

        self.event_engine
            .process(&SubscribeEvent::SubscriptionChanged {
                channels: inputs.channels(),
                channel_groups: inputs.channel_groups(),
            });
    }

    fn restore_subscription(&self, cursor: u64) {
        let inputs = self.current_input();

        // TODO: Uncomment after contract test server fix.
        // #[cfg(feature = "presence")]
        // if !inputs.is_empty {
        //     self.heartbeat_call.as_ref()(inputs.channels(), inputs.channel_groups());
        // }

        self.event_engine
            .process(&SubscribeEvent::SubscriptionRestored {
                channels: inputs.channels(),
                channel_groups: inputs.channel_groups(),
                cursor: SubscribeCursor {
                    timetoken: cursor.to_string(),
                    region: 0,
                },
            });
    }

    pub(crate) fn current_input(&self) -> SubscribeInput {
        self.subscribers.iter().fold(
            SubscribeInput::new(&None, &None),
            |mut input, subscription| {
                input += subscription.input.clone();
                input
            },
        )
    }
}

impl Debug for SubscriptionManagerRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubscriptionManagerRef {{\n\tevent_engine: {:?}\n\tsubscribers: {:?}\n}}",
            self.event_engine, self.subscribers
        )
    }
}

impl Drop for SubscriptionManagerRef {
    fn drop(&mut self) {
        self.unregister_all();
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::RequestRetryPolicy,
        dx::subscribe::{
            event_engine::{SubscribeEffectHandler, SubscribeState},
            result::SubscribeResult,
            subscription::SubscriptionBuilder,
            types::Message,
        },
        lib::alloc::sync::Arc,
        providers::futures_tokio::RuntimeTokio,
    };
    use spin::RwLock;

    fn event_engine() -> Arc<SubscribeEventEngine> {
        let (cancel_tx, _) = async_channel::bounded(1);

        SubscribeEventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |_| {
                    Box::pin(async move {
                        Ok(SubscribeResult {
                            cursor: Default::default(),
                            messages: Default::default(),
                        })
                    })
                }),
                Arc::new(|_| {
                    // Do nothing yet
                }),
                Arc::new(Box::new(|_| {
                    // Do nothing yet
                })),
                RequestRetryPolicy::None,
                cancel_tx,
            ),
            SubscribeState::Unsubscribed,
            RuntimeTokio,
        )
    }

    #[tokio::test]
    async fn register_subscription() {
        let mut manager = SubscriptionManager::new(
            event_engine(),
            Arc::new(|channels, _| {
                assert!(channels.is_some());
                assert_eq!(channels.unwrap().len(), 1);
            }),
            Arc::new(|_, _| {}),
        );
        let dummy_manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(dummy_manager)))),
            ..Default::default()
        }
        .channels(["test".into()])
        .execute()
        .unwrap();

        manager.register(subscription);

        assert_eq!(manager.subscribers.len(), 1);
    }

    #[tokio::test]
    async fn unregister_subscription() {
        let mut manager = SubscriptionManager::new(
            event_engine(),
            Arc::new(|_, _| {}),
            Arc::new(|channels, _| {
                assert!(channels.is_some());
                assert_eq!(channels.unwrap().len(), 1);
            }),
        );
        let dummy_manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(dummy_manager)))),
            ..Default::default()
        }
        .channels(["test".into()])
        .execute()
        .unwrap();

        manager.register(subscription.clone());
        manager.unregister(subscription);

        assert_eq!(manager.subscribers.len(), 0);
    }

    #[tokio::test]
    async fn notify_subscription_about_statuses() {
        let mut manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));
        let dummy_manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(dummy_manager)))),
            ..Default::default()
        }
        .channels(["test".into()])
        .execute()
        .unwrap();

        manager.register(subscription.clone());
        manager.notify_new_status(&SubscribeStatus::Connected);

        use futures::StreamExt;
        assert_eq!(
            subscription
                .status_stream()
                .next()
                .await
                .iter()
                .next()
                .unwrap(),
            &SubscribeStatus::Connected
        );
    }

    #[tokio::test]
    async fn notify_subscription_about_updates() {
        let mut manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));
        let dummy_manager =
            SubscriptionManager::new(event_engine(), Arc::new(|_, _| {}), Arc::new(|_, _| {}));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(dummy_manager)))),
            ..Default::default()
        }
        .channels(["test".into()])
        .execute()
        .unwrap();

        manager.register(subscription.clone());

        manager.notify_new_messages(vec![Update::Message(Message {
            channel: "test".into(),
            subscription: "test".into(),
            ..Default::default()
        })]);

        use futures::StreamExt;
        assert!(subscription.message_stream().next().await.is_some());
    }
}
