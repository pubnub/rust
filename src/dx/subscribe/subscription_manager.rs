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
use async_channel::Sender;
use spin::RwLock;

use super::event_engine::{SubscribeEffect, SubscribeEvent};

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
    subscribe_event_engine: RwLock<SubscribeEventEngine>,

    /// List of registered subscribers.
    ///
    /// List of subscribers which will receive real-time updates.
    pub subscribers: RwLock<Vec<Subscription>>,

    /// Event sender.
    ///
    /// Sender which will be used to send events to event engine.
    pub event_sender: Sender<SubscribeEvent>,
}

#[allow(dead_code)]
impl SubscriptionManager {
    pub fn new(
        subscribe_event_engine: SubscribeEventEngine,
        event_sender: Sender<SubscribeEvent>,
    ) -> Self {
        Self {
            subscribe_event_engine: RwLock::new(subscribe_event_engine),
            subscribers: Default::default(),
            event_sender,
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
    }

    pub fn unregister(&self, subscription: Subscription) {
        let mut subscribers_slot = self.subscribers.write();
        if let Some(position) = subscribers_slot
            .iter()
            .position(|val| val.id.eq(&subscription.id))
        {
            subscribers_slot.swap_remove(position);
        }
    }

    pub fn handle_event(&self, event: SubscribeEvent) -> Vec<Arc<SubscribeEffect>> {
        self.subscribe_event_engine.write().process(&event)
    }
}

#[cfg(test)]
mod should {
    use crate::core::RequestRetryPolicy;
    use core::sync::atomic::{AtomicBool, Ordering};

    use crate::dx::subscribe::result::SubscribeResult;
    use crate::lib::alloc::sync::Arc;

    use crate::dx::subscribe::event_engine::{SubscribeEffectHandler, SubscribeState};

    use super::*;

    #[allow(dead_code)]
    fn event_engine(processed: Arc<AtomicBool>) -> SubscribeEventEngine {
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
        )
    }

    #[tokio::test]
    async fn feed_event_engine_with_events() {
        //        let processed = Arc::new(AtomicBool::new(false));
        //        let sut = SubscriptionManager::new(event_engine(processed.clone()));
        //        sut.event_queue
        //            .write()
        //            .push_back(SubscribeEvent::SubscriptionChanged {
        //                channels: Some(vec!["channel".into()]),
        //                channel_groups: None,
        //            });
        //
        //        sut.update().await.unwrap();
        //
        //        assert!(processed.load(Ordering::Relaxed));
    }
}
