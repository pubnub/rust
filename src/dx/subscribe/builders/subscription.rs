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
use futures::{FutureExt, Stream};
use spin::{RwLock, RwLockWriteGuard};
use uuid::Uuid;

/// Regular messages stream.
#[derive(Debug)]
pub struct SubscriptionMessagesStream {
    inner: Arc<SubscriptionMessagesStreamRef>,
}

impl Deref for SubscriptionMessagesStream {
    type Target = SubscriptionMessagesStreamRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubscriptionMessagesStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Subscription message stream is not unique")
    }
}

impl Clone for SubscriptionMessagesStream {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Regular messages stream.
#[derive(Debug, Default)]
pub struct SubscriptionMessagesStreamRef {
    /// List of updates to be delivered to stream listener.
    updates: RwLock<Vec<Update>>,

    /// Subscription messages stream waker.
    ///
    /// Handler used each time when new data available for a stream listener.
    waker: RwLock<Option<Waker>>,
}

/// Subscription status stream.
///
/// Stream delivers changes in subscription status:
/// * `connected` - client connected to real-time [`PubNub`] network.
/// * `disconnected` - client has been disconnected from real-time [`PubNub`] network.
/// * `connection error` - client was unable to subscribe to specified channels and groups
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Debug)]
pub struct SubscriptionStatusStream {
    inner: Arc<SubscriptionStatusStreamRef>,
}

impl Deref for SubscriptionStatusStream {
    type Target = SubscriptionStatusStreamRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubscriptionStatusStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Subscription status stream is not unique")
    }
}

impl Clone for SubscriptionStatusStream {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Subscription status stream.
///
/// Stream delivers changes in subscription status:
/// * `connected` - client connected to real-time [`PubNub`] network.
/// * `disconnected` - client has been disconnected from real-time [`PubNub`] network.
/// * `connection error` - client was unable to subscribe to specified channels and groups
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Debug, Default)]
pub struct SubscriptionStatusStreamRef {
    /// List of updates to be delivered to stream listener.
    updates: RwLock<Vec<SubscribeStatus>>,

    /// Subscription messages stream waker.
    ///
    /// Handler used each time when new data available for a stream listener.
    waker: RwLock<Option<Waker>>,
}

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
        setter(strip_option, into),
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

    /// Messages / updates stream.
    ///
    /// Stream used to deliver only real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "None"
    )]
    pub(in crate::dx::subscribe) updates_stream: Option<SubscriptionMessagesStream>,

    /// Status stream.
    ///
    /// Stream used to deliver only subscription status changes.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "None"
    )]
    pub(in crate::dx::subscribe) status_stream: Option<SubscriptionStatusStream>,
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
    /// Unsubscribed current subscription.
    ///
    /// Cancel current subscription and remove it from the list of active
    /// subscriptions.
    ///
    /// # Examples
    /// ```
    /// ```
    pub async fn unsubscribe(self) {
        self.subscription_manager
            .write()
            .as_ref()
            .map(|manager| manager.unregister(self.clone()));
    }

    /// Stream of all subscription updates.
    ///
    /// Stream is used to deliver following updates:
    /// * received messages / updates
    /// * changes in subscription status
    pub fn stream(&self) -> Self {
        self.clone()
    }

    /// Stream with message / updates.
    ///
    /// Stream will deliver filtered set of updates which include only messages.
    pub fn message_stream(&mut self) -> SubscriptionMessagesStream {
        if let Some(stream) = &self.updates_stream {
            stream.clone()
        } else {
            let stream = SubscriptionMessagesStream {
                inner: Arc::new(SubscriptionMessagesStreamRef {
                    ..Default::default()
                }),
            };

            self.updates_stream = Some(stream.clone());
            stream
        }
    }

    /// Stream with subscription status updates.
    ///
    /// Stream will deliver filtered set of updates which include only
    /// subscription status change.
    pub fn status_stream(&mut self) -> SubscriptionStatusStream {
        if let Some(stream) = &self.status_stream {
            stream.clone()
        } else {
            let stream = SubscriptionStatusStream {
                inner: Arc::new(SubscriptionStatusStreamRef {
                    ..Default::default()
                }),
            };

            self.status_stream = Some(stream.clone());
            stream
        }
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_messages(&self, messages: &Vec<Update>) {
        let mut updates_stream_updates_slot: Option<RwLockWriteGuard<Vec<Update>>> = None;
        let mut updates_slot = self.updates.write();
        let updates_len = updates_slot.len();

        if let Some(updates_stream) = &self.updates_stream {
            updates_stream_updates_slot = Some(updates_stream.updates.write());
        }

        messages
            .iter()
            .filter(|msg| {
                self.channels.contains(&msg.channel())
                    || self.channel_groups.contains(&msg.channel_group())
            })
            .for_each(|msg| {
                if let Some(updates_stream_slot) = updates_stream_updates_slot.as_mut() {
                    updates_stream_slot.push(msg.clone());
                }

                updates_slot.push(SubscribeStreamEvent::Update(msg.clone()))
            });

        if updates_len < updates_slot.len() {
            self.wake_task();

            if let Some(updates_stream) = &self.updates_stream {
                updates_stream.wake_task();
            }
        }
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_status(&self, status: SubscribeStatus) {
        let mut updates_slot = self.updates.write();
        updates_slot.push(SubscribeStreamEvent::Status(status));

        self.wake_task();

        if let Some(status_stream) = &self.status_stream {
            let mut updates_slot = status_stream.updates.write();
            updates_slot.push(status);

            status_stream.wake_task();
        }
    }

    fn wake_task(&self) {
        let mut waker_slot = self.waker.write();
        if let Some(waker) = waker_slot.take() {
            waker.wake();
        }
    }
}

impl SubscriptionMessagesStream {
    fn wake_task(&self) {
        let mut waker_slot = self.waker.write();
        if let Some(waker) = waker_slot.take() {
            waker.wake();
        }
    }
}

impl SubscriptionStatusStream {
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

impl Stream for SubscriptionMessagesStream {
    type Item = Vec<Update>;

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

impl Stream for SubscriptionStatusStream {
    type Item = Vec<SubscribeStatus>;

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
//
// impl Subscription {
//     pub(crate) fn subscribe() -> Self {
//         // // TODO: implementation is a part of the different task
//         // let handshake: HandshakeFunction = |&_, &_, _, _| Ok(vec![]);
//         // let receive: ReceiveFunction = |&_, &_, &_, _, _| Ok(vec![]);
//         //
//         // Self {
//         //     engine: SubscribeEngine::new(
//         //         SubscribeEffectHandler::new(handshake, receive),
//         //         SubscribeState::Unsubscribed,
//         //     ),
//         // }
//         Self { /* fields */ }
//     }
// }

#[cfg(test)]
mod should {}
