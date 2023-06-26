//! Subscriptions' manager.
//!
//! This module contains manager which is responsible for tracking and updating
//! active subscription streams.
use crate::dx::subscribe::event_engine::SubscribeEventEngine;
use crate::dx::subscribe::result::Update;
use crate::dx::subscribe::types::SubscribeStreamEvent;
use crate::dx::subscribe::SubscribeStatus;
use crate::{
    dx::subscribe::subscription::Subscription,
    lib::alloc::{sync::Arc, vec::Vec},
};
use spin::RwLock;
use std::sync::mpsc::{channel, Sender};

/// Active subscriptions manager.
///
/// [`PubNubClient`] allows to have multiple [`subscription`] objects which will
/// be used to deliver real-time updates on channels and groups specified during
/// [`subscribe`] method call.
///
/// [`subscription`]: crate::Subscription
/// [`PubNubClient`]: crate::PubNubClient
#[derive(Debug)]
pub(crate) struct SubscriptionManager<Transport> {
    /// Subscription event engine.
    ///
    /// State machine which is responsible for subscription loop maintenance.
    subscribe_event_engine: RwLock<SubscribeEventEngine<Transport>>,

    /// List of registered subscribers.
    ///
    /// List of subscribers which will receive real-time updates.
    pub subscribers: RwLock<Vec<Arc<Subscription<Transport>>>>,
}

impl<Transport> SubscriptionManager<Transport> {
    pub fn new(subscribe_event_engine: SubscribeEventEngine<Transport>) -> Self {
        Self {
            subscribe_event_engine: RwLock::new(subscribe_event_engine),
            subscribers: Default::default(),
        }
    }

    pub fn notify_new_status(&self, status: &SubscribeStatus) {
        self.subscribers.read().iter().for_each(|subscription| {
            subscription.notify_update(SubscribeStreamEvent::Status(status.clone()));
        });
    }

    pub fn notify_new_messages(&self, messages: Vec<Update>) {
        messages.iter().for_each(|update| {
            let channel = update.channel();
            self.subscribers.read().iter().for_each(|subscription| {
                if subscription.channels.contains(channel) {
                    subscription.notify_update(SubscribeStreamEvent::Update(update.clone()));
                }
            });
        });
    }

    pub fn register(&self, subscription: Arc<Subscription<Transport>>) {
        let mut subscribers_slot = self.subscribers.write();
        subscribers_slot.push(subscription);
    }

    pub fn unregister(&self, subscription: Arc<Subscription<Transport>>) {
        let mut subscribers_slot = self.subscribers.write();
        if let Some(position) = subscribers_slot
            .iter()
            .position(|val| val.id.eq(&subscription.id))
        {
            subscribers_slot.swap_remove(position);
        }
    }
}
