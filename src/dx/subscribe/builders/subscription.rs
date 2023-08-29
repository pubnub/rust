//! Subscription module.
//!
//! Subscription module is responsible for handling real-time updates from
//! PubNub. It is responsible for handshake and receiving messages.
//! It is also responsible for delivering messages to the user.

use derive_builder::Builder;
use futures::Stream;
use spin::RwLock;
use uuid::Uuid;

use crate::subscribe::event_engine::SubscribeInput;
use crate::{
    core::PubNubError,
    dx::subscribe::{result::Update, types::SubscribeStreamEvent, SubscribeStatus},
    lib::{
        alloc::{
            collections::VecDeque,
            string::{String, ToString},
            sync::Arc,
            vec::Vec,
        },
        core::{
            fmt::{Debug, Formatter},
            ops::{Deref, DerefMut},
            pin::Pin,
            task::{Context, Poll, Waker},
        },
    },
    subscribe::SubscriptionManager,
};

/// Subscription stream.
///
/// Stream delivers changes in subscription status:
/// * `connected` - client connected to real-time [`PubNub`] network.
/// * `disconnected` - client has been disconnected from real-time [`PubNub`]
///   network.
/// * `connection error` - client was unable to subscribe to specified channels
///   and groups
///
/// and regular messages / signals.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Debug)]
pub struct SubscriptionStream<D> {
    inner: Arc<SubscriptionStreamRef<D>>,
}

impl<D> Deref for SubscriptionStream<D> {
    type Target = SubscriptionStreamRef<D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D> DerefMut for SubscriptionStream<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Subscription stream is not unique")
    }
}

impl<D> Clone for SubscriptionStream<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Subscription stream.
///
/// Stream delivers changes in subscription status:
/// * `connected` - client connected to real-time [`PubNub`] network.
/// * `disconnected` - client has been disconnected from real-time [`PubNub`]
///   network.
/// * `connection error` - client was unable to subscribe to specified channels
///   and groups
///
/// and regular messages / signals.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Debug, Default)]
pub struct SubscriptionStreamRef<D> {
    /// Update to be delivered to stream listener.
    updates: RwLock<VecDeque<D>>,

    /// Subscription stream waker.
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
#[derive(Builder)]
#[builder(
    pattern = "owned",
    name = "SubscriptionBuilder",
    build_fn(private, name = "build_internal", validate = "Self::validate"),
    no_std
)]
pub struct SubscriptionRef {
    /// Subscription module configuration.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom, strip_option)
    )]
    pub(in crate::dx::subscribe) subscription: Arc<RwLock<Option<SubscriptionManager>>>,

    /// User input with channels and groups.
    ///
    /// Object contains list of channels and channel groups on which
    /// [`PubNubClient`] will subscribe and notify about received real-time
    /// updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "SubscribeInput::new(&None, &None)"
    )]
    pub(in crate::dx::subscribe) input: SubscribeInput,

    /// Time cursor.
    ///
    /// Cursor used by subscription loop to identify point in time after
    /// which updates will be delivered.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Default::default()"
    )]
    pub(in crate::dx::subscribe) cursor: Option<u64>,

    /// Heartbeat interval.
    ///
    /// Interval in seconds that informs the server that the client should
    /// be considered alive.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Some(300)"
    )]
    pub(in crate::dx::subscribe) heartbeat: Option<u32>,

    /// Expression used to filter received messages.
    ///
    /// Expression used to filter received messages before they are delivered
    /// to the client.
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
        default = "RwLock::new(VecDeque::with_capacity(100))"
    )]
    pub(in crate::dx::subscribe) updates: RwLock<VecDeque<SubscribeStreamEvent>>,

    /// General subscription stream.
    ///
    /// Stream used to deliver all real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "RwLock::new(None)"
    )]
    pub(in crate::dx::subscribe) stream: RwLock<Option<SubscriptionStream<SubscribeStreamEvent>>>,

    /// Messages / updates stream.
    ///
    /// Stream used to deliver only real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "RwLock::new(None)"
    )]
    pub(in crate::dx::subscribe) updates_stream: RwLock<Option<SubscriptionStream<Update>>>,

    /// Status stream.
    ///
    /// Stream used to deliver only subscription status changes.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "RwLock::new(None)"
    )]
    pub(in crate::dx::subscribe) status_stream: RwLock<Option<SubscriptionStream<SubscribeStatus>>>,
}

impl SubscriptionBuilder {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let input = self
            .input
            .as_ref()
            .unwrap_or_else(|| panic!("Subscription input should be set by default"));

        if input.is_empty {
            return Err("Either channels or channel groups should be provided".into());
        }

        Ok(())
    }
    /// Channels from which real-time updates should be received.
    ///
    /// List of channels on which [`PubNubClient`] will subscribe and notify
    /// about received real-time updates.
    pub fn channels<L>(mut self, channels: L) -> Self
    where
        L: Into<Vec<String>>,
    {
        let user_input = SubscribeInput::new(&Some(channels.into()), &None);
        if let Some(input) = self.input {
            self.input = Some(input + user_input)
        } else {
            self.input = Some(user_input);
        }

        self
    }

    /// Channel groups from which real-time updates should be received.
    ///
    /// List of groups of channels on which [`PubNubClient`] will subscribe and
    /// notify about received real-time updates.
    pub fn channel_groups<L>(mut self, channel_groups: L) -> Self
    where
        L: Into<Vec<String>>,
    {
        let user_input = SubscribeInput::new(&None, &Some(channel_groups.into()));
        if let Some(input) = self.input {
            self.input = Some(input + user_input)
        } else {
            self.input = Some(user_input);
        }

        self
    }
}

impl SubscriptionBuilder {
    /// Construct subscription object.
    pub fn execute(self) -> Result<Subscription, PubNubError> {
        self.build_internal()
            .map(|subscription| Subscription {
                inner: Arc::new(subscription),
            })
            .map(|subscription| {
                if let Some(manager) = subscription.subscription.write().as_mut() {
                    manager.register(subscription.clone())
                }
                subscription
            })
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl Debug for SubscriptionRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> crate::lib::core::fmt::Result {
        write!(
            f,
            "Subscription {{ \nchannels: {:?}, \nchannel-groups: {:?}, \ncursor: {:?}, \nheartbeat: {:?}, \nfilter_expression: {:?}}}",
            self.input.channels(), self.input.channel_groups(), self.cursor, self.heartbeat, self.filter_expression
        )
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
        if let Some(manager) = self.subscription.write().as_mut() {
            manager.unregister(self.clone())
        }
    }

    /// Stream of all subscription updates.
    ///
    /// Stream is used to deliver following updates:
    /// * received messages / updates
    /// * changes in subscription status
    pub fn stream(&self) -> SubscriptionStream<SubscribeStreamEvent> {
        let mut stream = self.stream.write();

        if let Some(stream) = stream.clone() {
            stream
        } else {
            let events_stream = {
                let mut updates = self.updates.write();
                let stream = SubscriptionStream::new(updates.clone());
                updates.clear();
                stream
            };

            *stream = Some(events_stream.clone());

            events_stream
        }
    }

    /// Stream with message / updates.
    ///
    /// Stream will deliver filtered set of updates which include only messages.
    pub fn message_stream(&self) -> SubscriptionStream<Update> {
        let mut stream = self.updates_stream.write();

        if let Some(stream) = stream.clone() {
            stream
        } else {
            let events_stream = {
                let mut updates = self.updates.write();
                let updates_len = updates.len();
                let stream_updates = updates.iter().fold(
                    VecDeque::<Update>::with_capacity(updates_len),
                    |mut acc, event| {
                        if let SubscribeStreamEvent::Update(update) = event {
                            acc.push_back(update.clone());
                        }
                        acc
                    },
                );

                let stream = SubscriptionStream::new(stream_updates);
                updates.clear();

                stream
            };

            *stream = Some(events_stream.clone());
            events_stream
        }
    }

    /// Stream with subscription status updates.
    ///
    /// Stream will deliver filtered set of updates which include only
    /// subscription status change.
    pub fn status_stream(&self) -> SubscriptionStream<SubscribeStatus> {
        let mut stream = self.status_stream.write();

        if let Some(stream) = stream.clone() {
            stream
        } else {
            let events_stream = {
                let mut updates = self.updates.write();
                let updates_len = updates.len();
                let stream_statuses = updates.iter().fold(
                    VecDeque::<SubscribeStatus>::with_capacity(updates_len),
                    |mut acc, event| {
                        if let SubscribeStreamEvent::Status(update) = event {
                            acc.push_back(update.clone());
                        }
                        acc
                    },
                );

                let stream = SubscriptionStream::new(stream_statuses);
                updates.clear();

                stream
            };

            *stream = Some(events_stream.clone());
            events_stream
        }
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_messages(&self, messages: &[Update]) {
        // Filter out updates for this subscriber.
        let messages = messages
            .iter()
            .cloned()
            .filter(|update| self.subscribed_for_update(update))
            .collect::<Vec<Update>>();

        let common_stream = self.stream.read();
        let stream = self.updates_stream.read();
        let accumulate = common_stream.is_none() && stream.is_none();

        if accumulate {
            let mut updates_slot = self.updates.write();
            updates_slot.extend(messages.into_iter().map(SubscribeStreamEvent::Update));
        } else {
            if let Some(stream) = common_stream.clone() {
                let mut updates_slot = stream.updates.write();
                let updates_len = updates_slot.len();
                updates_slot.extend(
                    messages
                        .clone()
                        .into_iter()
                        .map(SubscribeStreamEvent::Update),
                );
                updates_slot
                    .len()
                    .ne(&updates_len)
                    .then(|| stream.wake_task());
            }

            if let Some(stream) = stream.clone() {
                let mut updates_slot = stream.updates.write();
                let updates_len = updates_slot.len();
                updates_slot.extend(messages);
                updates_slot
                    .len()
                    .ne(&updates_len)
                    .then(|| stream.wake_task());
            }
        }
    }

    /// Handle received real-time updates.
    pub(in crate::dx::subscribe) fn handle_status(&self, status: SubscribeStatus) {
        let common_stream = self.stream.read();
        let stream = self.status_stream.read();
        let accumulate = common_stream.is_none() && stream.is_none();

        if accumulate {
            let mut updates_slot = self.updates.write();
            updates_slot.push_back(SubscribeStreamEvent::Status(status));
        } else {
            if let Some(stream) = common_stream.clone() {
                let mut updates_slot = stream.updates.write();
                let updates_len = updates_slot.len();
                updates_slot.push_back(SubscribeStreamEvent::Status(status.clone()));
                updates_slot
                    .len()
                    .ne(&updates_len)
                    .then(|| stream.wake_task());
            }

            if let Some(stream) = stream.clone() {
                let mut updates_slot = stream.updates.write();
                let updates_len = updates_slot.len();
                updates_slot.push_back(status.clone());
                updates_slot
                    .len()
                    .ne(&updates_len)
                    .then(|| stream.wake_task());
            }
        }
    }

    fn subscribed_for_update(&self, update: &Update) -> bool {
        let subscription = &update.subscription();
        self.input.contains_channel(subscription) || self.input.contains_channel_group(subscription)
    }
}

impl<D> SubscriptionStream<D> {
    fn new(updates: VecDeque<D>) -> Self {
        let mut stream_updates = VecDeque::with_capacity(100);
        stream_updates.extend(updates);

        Self {
            inner: Arc::new(SubscriptionStreamRef {
                updates: RwLock::new(stream_updates),
                waker: RwLock::new(None),
            }),
        }
    }

    fn wake_task(&self) {
        if let Some(waker) = self.waker.write().take() {
            waker.wake();
        }
    }
}

impl<D> Stream for SubscriptionStream<D> {
    type Item = D;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut waker_slot = self.waker.write();
        *waker_slot = Some(cx.waker().clone());

        if let Some(update) = self.updates.write().pop_front() {
            Poll::Ready(Some(update))
        } else {
            Poll::Pending
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
