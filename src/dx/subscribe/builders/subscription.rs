use core::ops::{Deref, DerefMut};
use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{
    core::PubNubError,
    dx::subscribe::{
        result::Update, subscription_manager::SubscriptionManager, types::SubscribeStreamEvent,
        SubscribeCursor, SubscribeStatus,
    },
    lib::alloc::{
        string::{String, ToString},
        sync::Arc,
        vec::Vec,
    },
};
use derive_builder::Builder;
use futures::Stream;
use spin::RwLock;
use uuid::Uuid;

/// Subscription that is responsible for getting messages from PubNub.
///
/// Subscription provides a way to get messages from PubNub. It is responsible
/// for handshake and receiving messages.
#[derive(Debug)]
pub struct Subscription {
    inner: Arc<SubscriptionRef>,
}

impl Deref for Subscription {
    type Target = SubscriptionRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Subscription {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Subscription is not unique")
    }
}

impl Clone for Subscription {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Subscription that is responsible for getting messages from PubNub.
///
/// Subscription provides a way to get messages from PubNub. It is responsible
/// for handshake and receiving messages.
///
/// It should not be created directly, but via [`PubNubClient::subscribe`]
/// and wrapped in [`Subscription`] struct.
#[derive(Builder, Debug)]
#[builder(
    pattern = "owned",
    name = "SubscriptionBuilder",
    build_fn(private, name = "build_internal", validate = "Self::validate"),
    no_std
)]
#[allow(dead_code)]
pub struct SubscriptionRef {
    /// Manager of active subscriptions.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom, strip_option)
    )]
    pub(in crate::dx::subscribe) subscription_manager: Arc<RwLock<Option<SubscriptionManager>>>,

    /// Channels from which real-time updates should be received.
    ///
    /// List of channels on which [`PubNubClient`] will subscribe and notify
    /// about received real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(into, strip_option),
        default = "Vec::new()"
    )]
    pub(in crate::dx::subscribe) channels: Vec<String>,

    /// Channel groups from which real-time updates should be received.
    ///
    /// List of groups of channels on which [`PubNubClient`] will subscribe and
    /// notify about received real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(into, strip_option),
        default = "Vec::new()"
    )]
    pub(in crate::dx::subscribe) channel_groups: Vec<String>,

    /// Time cursor.
    ///
    /// Cursor used by subscription loop to identify point in time after
    /// which updates will be delivered.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Default::default()"
    )]
    pub(in crate::dx::subscribe) cursor: Option<SubscribeCursor>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Some(300)"
    )]
    pub(in crate::dx::subscribe) heartbeat: Option<u32>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "Uuid::new_v4().to_string()"
    )]
    pub(in crate::dx::subscribe) id: String,

    /// List of updates to be delivered to stream listener.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "RwLock::new(Vec::with_capacity(100))"
    )]
    pub(in crate::dx::subscribe) updates: RwLock<Vec<SubscribeStreamEvent>>,

    /// Subscription stream waker.
    ///
    /// Handler used each time when new data available for a stream listener.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "RwLock::new(None)"
    )]
    waker: RwLock<Option<Waker>>,
}

impl SubscriptionBuilder {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        if channels_len == groups_len && channels_len == 0 {
            Err("Either channels or channel groups should be provided".into())
        } else {
            Ok(())
        }
    }
}

impl SubscriptionBuilder {
    /// Construct subscription object.
    pub fn build(self) -> Result<Subscription, PubNubError> {
        self.build_internal()
            .map(|subscription| Subscription {
                inner: Arc::new(subscription),
            })
            .map(|subscription| {
                subscription
                    .subscription_manager
                    .write()
                    .as_ref()
                    .map(|manager| manager.register(subscription.clone()));
                subscription
            })
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl Subscription {
    /// Cancel current subscription.
    ///
    /// Cancel current subscription and remove it from the list of active
    /// subscriptions.
    ///
    /// # Examples
    /// ```
    /// ```
    pub async fn cancel(&self) {
        self.subscription_manager
            .write()
            .as_ref()
            .map(|manager| manager.unregister(self.clone()));
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_messages(&self, messages: &Vec<Update>) {
        let mut updates_slot = self.updates.write();
        let updates_len = updates_slot.len();

        messages
            .iter()
            .filter(|msg| {
                self.channels.contains(&msg.channel())
                    || self.channel_groups.contains(&msg.channel_group())
            })
            .for_each(|msg| updates_slot.push(SubscribeStreamEvent::Update(msg.clone())));

        if updates_len < updates_slot.len() {
            self.wake_task();
        }
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_status(&self, status: SubscribeStatus) {
        let mut updates_slot = self.updates.write();
        updates_slot.push(SubscribeStreamEvent::Status(status));

        self.wake_task();
    }

    fn wake_task(&self) {
        let mut waker_slot = self.waker.write();
        if let Some(waker) = waker_slot.take() {
            waker.wake();
        }
    }
}

impl Stream for Subscription {
    type Item = Vec<SubscribeStreamEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut updates_slot = self.updates.write();
        let mut waker_slot = self.waker.write();
        *waker_slot = Some(cx.waker().clone());

        if updates_slot.is_empty() {
            Poll::Pending
        } else {
            let updates = updates_slot.to_vec();
            updates_slot.clear();

            Poll::Ready(Some(updates))
        }
    }
}
