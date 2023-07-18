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

use spin::RwLock;

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
                channels: Some(channels),
                channel_groups: Some(channel_groups),
            });
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
}

#[cfg(test)]
mod should {
    use crate::core::RequestRetryPolicy;
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
}
