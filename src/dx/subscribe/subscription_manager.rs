//! Subscriptions' manager.
//!
//! This module contains manager which is responsible for tracking and updating
//! active subscription streams.

use crate::{
    dx::subscribe::{
        event_engine::SubscribeEventEngine, result::Update, subscription::Subscription,
        SubscribeStatus,
    },
    lib::alloc::{sync::Arc, vec::Vec},
};

use spin::{RwLock, RwLockWriteGuard};

use super::event_engine::SubscribeEvent;

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
    pub subscribers: RwLock<Vec<Subscription>>,
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
            subscription.handle_status(status.clone());
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
    use crate::core::RequestRetryPolicy;
    use crate::dx::subscribe::subscription::SubscriptionBuilder;
    use crate::providers::futures_tokio::TokioRuntime;
    use core::sync::atomic::{AtomicBool, Ordering};

    use crate::dx::subscribe::result::SubscribeResult;
    use crate::lib::alloc::sync::Arc;

    use crate::dx::subscribe::event_engine::{SubscribeEffectHandler, SubscribeState};

    use super::*;

    #[allow(dead_code)]
    fn event_engine(processed: Arc<AtomicBool>) -> Arc<SubscribeEventEngine> {
        let (cancel_tx, _) = async_channel::bounded(1);

        SubscribeEventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |_| {
                    processed.store(true, Ordering::Relaxed);
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
        let processed = Arc::new(AtomicBool::new(false));

        let event_engine = event_engine(processed.clone());
        let manager = Arc::new(RwLock::new(Some(SubscriptionManager::new(event_engine))));

        SubscriptionBuilder {
            subscription_manager: Some(manager.clone()),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        assert_eq!(manager.read().as_ref().unwrap().subscribers.read().len(), 1);
    }

    #[tokio::test]
    async fn unregister_subscription() {
        let processed = Arc::new(AtomicBool::new(false));

        let event_engine = event_engine(processed.clone());
        let manager = Arc::new(RwLock::new(Some(SubscriptionManager::new(event_engine))));

        let subscription = SubscriptionBuilder {
            subscription_manager: Some(manager.clone()),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        manager.read().as_ref().unwrap().unregister(subscription);

        assert_eq!(manager.read().as_ref().unwrap().subscribers.read().len(), 0);
    }

    #[tokio::test]
    #[ignore = "mutable reference claimed multiple times"]
    async fn notify_subscription_about_statuses() {
        let processed = Arc::new(AtomicBool::new(false));

        let event_engine = event_engine(processed.clone());
        let manager = Arc::new(RwLock::new(Some(SubscriptionManager::new(event_engine))));

        let subscription = SubscriptionBuilder {
            subscription_manager: Some(manager.clone()),
            ..Default::default()
        }
        .channels(["test".into()])
        .build()
        .unwrap();

        manager
            .read()
            .as_ref()
            .unwrap()
            .notify_new_status(&SubscribeStatus::Connected);

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
}
