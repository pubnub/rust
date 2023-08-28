//! Subscriptions' manager.
//!
//! This module contains manager which is responsible for tracking and updating
//! active subscription streams.

use super::event_engine::SubscribeEvent;
use crate::subscribe::SubscribeCursor;
use crate::{
    dx::subscribe::{
        event_engine::SubscribeEventEngine, result::Update, subscription::Subscription,
        SubscribeStatus,
    },
    lib::alloc::{sync::Arc, vec::Vec},
};

/// Active subscriptions manager.
///
/// [`PubNubClient`] allows to have multiple [`subscription`] objects which will
/// be used to deliver real-time updates on channels and groups specified during
/// [`subscribe`] method call.
///
/// [`subscription`]: crate::Subscription
/// [`PubNubClient`]: crate::PubNubClient
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SubscriptionManager {
    /// Subscription event engine.
    ///
    /// State machine which is responsible for subscription loop maintenance.
    subscribe_event_engine: Arc<SubscribeEventEngine>,

    /// List of registered subscribers.
    ///
    /// List of subscribers which will receive real-time updates.
    subscribers: Vec<Subscription>,
}

#[allow(dead_code)]
impl SubscriptionManager {
    pub fn new(subscribe_event_engine: Arc<SubscribeEventEngine>) -> Self {
        Self {
            subscribe_event_engine,
            subscribers: Default::default(),
        }
    }

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
            self.change_subscription();
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

        self.change_subscription();
    }

    fn change_subscription(&self) {
        let channels = self
            .subscribers
            .iter()
            .flat_map(|val| val.channels.iter())
            .cloned()
            .collect::<Vec<_>>();

        let channel_groups = self
            .subscribers
            .iter()
            .flat_map(|val| val.channel_groups.iter())
            .cloned()
            .collect::<Vec<_>>();

        self.subscribe_event_engine
            .process(&SubscribeEvent::SubscriptionChanged {
                channels: (!channels.is_empty()).then_some(channels),
                channel_groups: (!channel_groups.is_empty()).then_some(channel_groups),
            });
    }

    fn restore_subscription(&self, cursor: u64) {
        let channels = self
            .subscribers
            .iter()
            .flat_map(|val| val.channels.iter())
            .cloned()
            .collect::<Vec<_>>();

        let channel_groups = self
            .subscribers
            .iter()
            .flat_map(|val| val.channel_groups.iter())
            .cloned()
            .collect::<Vec<_>>();

        self.subscribe_event_engine
            .process(&SubscribeEvent::SubscriptionRestored {
                channels: (!channels.is_empty()).then_some(channels),
                channel_groups: (!channel_groups.is_empty()).then_some(channel_groups),
                cursor: SubscribeCursor {
                    timetoken: cursor.to_string(),
                    region: 0,
                },
            });
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
            SubscriptionConfiguration, SubscriptionConfigurationRef,
        },
        lib::alloc::sync::Arc,
        providers::futures_tokio::TokioRuntime,
    };
    use spin::RwLock;

    #[allow(dead_code)]
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
            TokioRuntime,
        )
    }

    #[tokio::test]
    async fn register_subscription() {
        let mut manager = SubscriptionManager::new(event_engine());
        let dummy_manager = SubscriptionManager::new(event_engine());

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: dummy_manager,
                    deserializer: None,
                }),
            })))),
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
        let mut manager = SubscriptionManager::new(event_engine());
        let dummy_manager = SubscriptionManager::new(event_engine());

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: dummy_manager,
                    deserializer: None,
                }),
            })))),
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
        let mut manager = SubscriptionManager::new(event_engine());
        let dummy_manager = SubscriptionManager::new(event_engine());

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: dummy_manager,
                    deserializer: None,
                }),
            })))),
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
        let mut manager = SubscriptionManager::new(event_engine());
        let dummy_manager = SubscriptionManager::new(event_engine());

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: dummy_manager,
                    deserializer: None,
                }),
            })))),
            ..Default::default()
        }
        .channels(["test".into()])
        .execute()
        .unwrap();

        manager.register(subscription.clone());

        manager.notify_new_messages(vec![Update::Message(Message {
            channel: "test".into(),
            ..Default::default()
        })]);

        use futures::StreamExt;
        assert!(subscription.message_stream().next().await.is_some());
    }
}
