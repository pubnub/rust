//! Subscriptions' manager.
//!
//! This module contains manager which is responsible for tracking and updating
//! active subscription streams.

use super::event_engine::SubscribeEvent;
use crate::{
    dx::subscribe::{
        event_engine::SubscribeEventEngine, result::Update, subscription::Subscription,
        SubscribeStatus,
    },
    lib::alloc::{sync::Arc, vec::Vec},
};
use spin::{RwLock, RwLockWriteGuard};

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
    subscribe_event_engine: RwLock<Arc<SubscribeEventEngine>>,

    /// List of registered subscribers.
    ///
    /// List of subscribers which will receive real-time updates.
    subscribers: RwLock<Vec<Subscription>>,
}

#[allow(dead_code)]
impl SubscriptionManager {
    pub fn new(subscribe_event_engine: Arc<SubscribeEventEngine>) -> Self {
        Self {
            subscribe_event_engine: RwLock::new(subscribe_event_engine),
            subscribers: Default::default(),
        }
    }

    pub fn notify_new_status(&self, status: &SubscribeStatus) {
        self.subscribers.read().iter().for_each(|subscription| {
            subscription.handle_status(*status);
        });
    }

    pub fn notify_new_messages(&self, messages: Vec<Update>) {
        self.subscribers.read().iter().for_each(|subscription| {
            subscription.handle_messages(&messages);
        });
    }

    pub fn register(&self, subscription: Subscription) {
        let mut subscribers_slot = self.subscribers.write();
        subscribers_slot.push(subscription);

        self.change_subscription(&subscribers_slot);
    }

    pub fn unregister(&self, subscription: Subscription) {
        let mut subscribers_slot = self.subscribers.write();
        if let Some(position) = subscribers_slot
            .iter()
            .position(|val| val.id.eq(&subscription.id))
        {
            subscribers_slot.swap_remove(position);
        }

        self.change_subscription(&subscribers_slot);
    }

    fn change_subscription(&self, subscribers_slot: &RwLockWriteGuard<Vec<Subscription>>) {
        let channels = subscribers_slot
            .iter()
            .flat_map(|val| val.channels.iter())
            .cloned()
            .collect::<Vec<_>>();

        let channel_groups = subscribers_slot
            .iter()
            .flat_map(|val| val.channel_groups.iter())
            .cloned()
            .collect::<Vec<_>>();

        self.subscribe_event_engine
            .write()
            .process(&SubscribeEvent::SubscriptionChanged {
                channels: (!channels.is_empty()).then_some(channels),
                channel_groups: (!channel_groups.is_empty()).then_some(channel_groups),
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
        let event_engine = event_engine();
        let manager = Arc::new(SubscriptionManager::new(event_engine));

        SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: manager.clone(),
                    deserializer: None,
                }),
            })))),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        assert_eq!(manager.subscribers.read().len(), 1);
    }

    #[tokio::test]
    async fn unregister_subscription() {
        let event_engine = event_engine();
        let manager = Arc::new(SubscriptionManager::new(event_engine));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: manager.clone(),
                    deserializer: None,
                }),
            })))),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        manager.unregister(subscription);

        assert_eq!(manager.subscribers.read().len(), 0);
    }

    #[tokio::test]
    async fn notify_subscription_about_statuses() {
        let event_engine = event_engine();
        let manager = Arc::new(SubscriptionManager::new(event_engine));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: manager.clone(),
                    deserializer: None,
                }),
            })))),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

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
        let event_engine = event_engine();
        let manager = Arc::new(SubscriptionManager::new(event_engine));

        let subscription = SubscriptionBuilder {
            subscription: Some(Arc::new(RwLock::new(Some(SubscriptionConfiguration {
                inner: Arc::new(SubscriptionConfigurationRef {
                    subscription_manager: manager.clone(),
                    deserializer: None,
                }),
            })))),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        manager.notify_new_messages(vec![Update::Message(Message {
            channel: "test".into(),
            ..Default::default()
        })]);

        use futures::StreamExt;
        assert!(subscription.message_stream().next().await.is_some());
    }
}
