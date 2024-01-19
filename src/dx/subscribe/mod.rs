//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

#[cfg(feature = "std")]
use futures::{
    future::{ready, BoxFuture},
    FutureExt,
};
#[cfg(feature = "std")]
use spin::RwLock;

#[cfg(feature = "std")]
use crate::{
    core::{Deserializer, PubNubError, Transport},
    lib::alloc::{boxed::Box, sync::Arc, vec::Vec},
    subscribe::result::SubscribeResult,
};

#[cfg(feature = "std")]
use crate::{
    core::{
        event_engine::{CancellationTask, EventEngine},
        runtime::Runtime,
        DataStream, PubNubEntity,
    },
    lib::alloc::string::ToString,
};

use crate::{
    dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String,
    subscribe::raw::RawSubscriptionBuilder,
};

#[cfg(all(feature = "presence", feature = "std"))]
use event_engine::SubscriptionInput;
#[cfg(feature = "std")]
use event_engine::{
    types::SubscriptionParams, SubscribeEffectHandler, SubscribeEventEngine, SubscribeState,
};

#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
pub(crate) mod event_engine;

#[cfg(feature = "std")]
pub(crate) use subscription_manager::SubscriptionManager;
#[cfg(feature = "std")]
pub(crate) mod subscription_manager;

#[cfg(feature = "std")]
#[doc(inline)]
pub(crate) use event_dispatcher::EventDispatcher;
#[cfg(feature = "std")]
mod event_dispatcher;

#[doc(inline)]
pub use types::*;
pub mod types;

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
pub use result::{SubscribeResponseBody, Update};
pub mod result;

#[cfg(feature = "std")]
#[doc(inline)]
pub use subscription::Subscription;
#[cfg(feature = "std")]
mod subscription;

#[cfg(feature = "std")]
#[doc(inline)]
pub use subscription_set::SubscriptionSet;
#[cfg(feature = "std")]
mod subscription_set;

#[cfg(feature = "std")]
#[doc(inline)]
pub use traits::{EventEmitter, EventSubscriber, Subscribable, SubscribableType, Subscriber};
#[cfg(feature = "std")]
pub(crate) mod traits;

#[cfg(feature = "std")]
impl<T, D> PubNubClientInstance<T, D> {
    /// Stream used to notify connection state change events.
    pub fn status_stream(&self) -> DataStream<ConnectionStatus> {
        self.event_dispatcher.status_stream()
    }

    /// Handle connection status change.
    ///
    /// # Arguments
    ///
    /// * `status` - Current connection status.
    pub(crate) fn handle_status(&self, status: ConnectionStatus) {
        self.event_dispatcher.handle_status(status)
    }

    /// Handles the given events.
    ///
    /// # Arguments
    ///
    /// * `cursor` - A time cursor for next portion of events.
    /// * `events` - A slice of real-time events from multiplexed subscription.
    pub(crate) fn handle_events(&self, cursor: SubscriptionCursor, events: &[Update]) {
        let mut cursor_slot = self.cursor.write();
        if let Some(current_cursor) = cursor_slot.as_ref() {
            cursor
                .gt(current_cursor)
                .then(|| *cursor_slot = Some(cursor));
        } else {
            *cursor_slot = Some(cursor);
        }

        self.event_dispatcher.handle_events(events.to_vec())
    }

    /// Creates a clone of the [`PubNubClientInstance`] with an empty event
    /// dispatcher.
    ///
    /// Empty clones have the same client state but an empty list of
    /// real-time event listeners, which makes it possible to attach listeners
    /// specific to the context. When the cloned client set goes out of scope,
    /// all associated listeners will be invalidated and released.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let client = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// // ...
    /// // We need to pass client into other component which would like to
    /// // have own listeners to handle real-time events.
    /// let empty_client = client.clone_empty();
    /// // self.other_component(empty_client);    
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// A new instance of the subscription object with an empty event
    /// dispatcher.
    pub fn clone_empty(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            cursor: Arc::clone(&self.cursor),
            event_dispatcher: Arc::clone(&self.event_dispatcher),
        }
    }
}

#[cfg(feature = "std")]
impl<T, D> PubNubClientInstance<T, D>
where
    T: Transport + Send + 'static,
    D: Deserializer + 'static,
{
    /// Creates multiplexed subscriptions.
    ///
    /// # Arguments
    ///
    /// * `entities` - A `Vec` of known subscribable entities.
    /// * `options` - Optional subscription options.
    ///
    /// # Returns
    ///
    /// The created [`SubscriptionSet`] object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use futures::StreamExt;
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset, subscribe::EventEmitter};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), pubnub::core::PubNubError> {
    /// use pubnub::subscribe::EventSubscriber;
    /// let client = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let subscription = client.subscription(
    ///     Some(&["my_channel_1", "my_channel_2", "my_channel_3"]),
    ///     None,
    ///     None,
    /// );
    /// // Message stream for handling real-time `Message` events.
    /// let stream = subscription.messages_stream();
    /// #     Ok(())
    /// # }
    /// ```
    pub fn subscription<N>(
        &self,
        channels: Option<&[N]>,
        channel_groups: Option<&[N]>,
        options: Option<Vec<SubscriptionOptions>>,
    ) -> Arc<SubscriptionSet<T, D>>
    where
        N: Into<String> + Clone,
    {
        let mut entities: Vec<PubNubEntity<T, D>> = vec![];
        if let Some(channel_names) = channels {
            entities.extend(
                channel_names
                    .iter()
                    .cloned()
                    .map(|name| self.create_channel(name).into())
                    .collect::<Vec<PubNubEntity<T, D>>>(),
            );
        }
        if let Some(channel_group_names) = channel_groups {
            entities.extend(
                channel_group_names
                    .iter()
                    .cloned()
                    .map(|name| self.create_channel_group(name).into())
                    .collect::<Vec<PubNubEntity<T, D>>>(),
            );
        }

        SubscriptionSet::new(entities, options)
    }

    /// Stop receiving real-time updates.
    ///
    /// Stop receiving real-time updates for previously subscribed channels and
    /// groups by temporarily disconnecting from the [`PubNub`] network.
    ///
    /// ```no_run
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{EventEmitter, SubscribeStreamEvent, Update};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// #     let client = PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #             subscribe_key: "demo",
    /// #             publish_key: Some("demo"),
    /// #             secret_key: None,
    /// #         })
    /// #         .with_user_id("user_id")
    /// #         .build()?;
    /// # let subscription = client.subscription(Some(&["channel"]), None, None);
    /// # let stream = // DataStream<Message>
    /// #     subscription.messages_stream();
    /// client.disconnect();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub`]: https://www.pubnub.com
    pub fn disconnect(&self) {
        #[cfg(feature = "presence")]
        let mut input: Option<SubscriptionInput> = None;

        if let Some(manager) = self.subscription_manager().read().as_ref() {
            #[cfg(feature = "presence")]
            {
                let current_input = manager.current_input();
                input = (!current_input.is_empty).then_some(current_input);
            }

            manager.disconnect()
        }

        #[cfg(feature = "presence")]
        {
            let Some(input) = input else {
                return;
            };

            if self.config.presence.heartbeat_interval.is_none() {
                let mut request = self.leave();
                if let Some(channels) = input.channels() {
                    request = request.channels(channels);
                }
                if let Some(channel_groups) = input.channel_groups() {
                    request = request.channel_groups(channel_groups);
                }

                self.runtime.spawn(async {
                    let _ = request.execute().await;
                })
            } else if let Some(presence) = self.presence_manager().read().as_ref() {
                presence.disconnect();
            }
        }
    }

    /// Resume real-time updates receiving.
    ///
    /// Restore real-time updates receive from previously subscribed channels
    /// and groups by restoring connection to the [`PubNub`] network.
    ///
    /// ```no_run
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{EventEmitter, SubscribeStreamEvent, Update};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// #     let client = PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #             subscribe_key: "demo",
    /// #             publish_key: Some("demo"),
    /// #             secret_key: None,
    /// #         })
    /// #         .with_user_id("user_id")
    /// #         .build()?;
    /// # let subscription = client.subscription(Some(&["channel"]), None, None);
    /// # let stream = // DataStream<Message>
    /// #     subscription.messages_stream();
    /// # // .....
    /// # client.disconnect();
    /// client.reconnect(None);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub`]: https://www.pubnub.com
    pub fn reconnect(&self, cursor: Option<SubscriptionCursor>) {
        #[cfg(feature = "presence")]
        let mut input: Option<SubscriptionInput> = None;

        if let Some(manager) = self.subscription_manager().read().as_ref() {
            #[cfg(feature = "presence")]
            {
                let current_input = manager.current_input();
                input = (!current_input.is_empty).then_some(current_input);
            }
            manager.reconnect(cursor);
        }

        #[cfg(feature = "presence")]
        {
            let Some(input) = input else {
                return;
            };

            if self.config.presence.heartbeat_interval.is_none() {
                let mut request = self.heartbeat();
                if let Some(channels) = input.channels() {
                    request = request.channels(channels);
                }
                if let Some(channel_groups) = input.channel_groups() {
                    request = request.channel_groups(channel_groups);
                }

                self.runtime.spawn(async {
                    let _ = request.execute().await;
                })
            } else if let Some(presence) = self.presence_manager().read().as_ref() {
                presence.reconnect();
            }
        }
    }

    /// Unsubscribes from all real-time events.
    ///
    /// Stop any actions for receiving real-time events processing for all
    /// created [`Subscription`] and [`SubscriptionSet`].
    pub fn unsubscribe_all(&self) {
        {
            if let Some(manager) = self.subscription_manager().write().as_mut() {
                manager.unregister_all()
            }
        }
    }

    /// Subscription manager which maintains Subscription EE.
    ///
    /// # Returns
    ///
    /// Returns an [`SubscriptionManager`] which represents the manager.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn subscription_manager(&self) -> Arc<RwLock<Option<SubscriptionManager<T, D>>>> {
        {
            // Initialize subscription module when it will be first required.
            let mut slot = self.subscription.write();
            if slot.is_none() {
                #[cfg(feature = "presence")]
                let heartbeat_self = self.clone();
                #[cfg(feature = "presence")]
                let leave_self = self.clone();

                *slot = Some(SubscriptionManager::new(
                    self.subscribe_event_engine(),
                    #[cfg(feature = "presence")]
                    Arc::new(move |channels, groups| {
                        Self::subscribe_heartbeat_call(heartbeat_self.clone(), channels, groups);
                    }),
                    #[cfg(feature = "presence")]
                    Arc::new(move |channels, groups| {
                        Self::subscribe_leave_call(leave_self.clone(), channels, groups);
                    }),
                ));
            }
        }

        self.subscription.clone()
    }

    fn subscribe_event_engine(&self) -> Arc<SubscribeEventEngine> {
        let channel_bound = 10; // TODO: Think about this value
        let emit_messages_client = self.clone();
        let emit_status_client = self.clone();
        let subscribe_client = self.clone();
        let request_retry = self.config.transport.retry_configuration.clone();
        let request_subscribe_retry = request_retry.clone();
        let runtime = self.runtime.clone();
        let runtime_sleep = runtime.clone();
        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);

        EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |params| {
                    let delay_in_microseconds = request_subscribe_retry.retry_delay(
                        Some("/v2/subscribe".to_string()),
                        &params.attempt,
                        params.reason.as_ref(),
                    );
                    let inner_runtime_sleep = runtime_sleep.clone();

                    Self::subscribe_call(
                        subscribe_client.clone(),
                        params.clone(),
                        Arc::new(move || {
                            if let Some(delay) = delay_in_microseconds {
                                inner_runtime_sleep
                                    .clone()
                                    .sleep_microseconds(delay)
                                    .boxed()
                            } else {
                                ready(()).boxed()
                            }
                        }),
                        cancel_rx.clone(),
                    )
                }),
                Arc::new(move |status| Self::emit_status(emit_status_client.clone(), &status)),
                Arc::new(Box::new(move |updates, cursor: SubscriptionCursor| {
                    Self::emit_messages(emit_messages_client.clone(), updates, cursor)
                })),
                request_retry,
                cancel_tx,
            ),
            SubscribeState::Unsubscribed,
            runtime,
        )
    }

    fn subscribe_call<F>(
        client: Self,
        params: SubscriptionParams,
        delay: Arc<F>,
        cancel_rx: async_channel::Receiver<String>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let mut request = client
            .subscribe_request()
            .cursor(params.cursor.cloned().unwrap_or_default()); // TODO: is this clone required?

        if let Some(channels) = params.channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = params.channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }

        #[cfg(feature = "presence")]
        {
            let state = client.state.read();
            if params.cursor.is_none() && !state.is_empty() {
                request = request.state(state.clone());
            }
        }

        let cancel_task = CancellationTask::new(cancel_rx, params.effect_id.to_owned()); // TODO: needs to be owned?

        request
            .execute_with_cancel_and_delay(delay, cancel_task)
            .boxed()
    }

    /// Subscription event engine presence `join` announcement.
    ///
    /// The heartbeat call method provides few different flows based on the
    /// presence event engine state:
    /// * can operate - call `join` announcement
    /// * can't operate (heartbeat interval not set) - make direct `heartbeat`
    ///   call.
    #[cfg(all(feature = "presence", feature = "std"))]
    fn subscribe_heartbeat_call(
        client: Self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        let channels = Self::presence_filtered_entries(channels);
        let channel_groups = Self::presence_filtered_entries(channel_groups);

        if let Some(presence) = client.presence_manager().read().as_ref() {
            presence.announce_join(channels, channel_groups);
        }
    }

    /// Subscription event engine presence `leave` announcement.
    ///
    /// The leave call method provides few different flows based on the
    /// presence event engine state:
    /// * can operate - call `leave` announcement
    /// * can't operate (heartbeat interval not set) - make direct `leave` call.
    #[cfg(feature = "presence")]
    fn subscribe_leave_call(
        client: Self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        #[cfg(feature = "presence")]
        {
            let channels = Self::presence_filtered_entries(channels);
            let channel_groups = Self::presence_filtered_entries(channel_groups);

            if let Some(presence) = client.presence_manager().read().as_ref() {
                presence.announce_left(channels, channel_groups);
            }
        }
    }

    fn emit_status(client: Self, status: &ConnectionStatus) {
        if let Some(manager) = client.subscription_manager().read().as_ref() {
            manager.notify_new_status(status)
        }
    }

    fn emit_messages(client: Self, messages: Vec<Update>, cursor: SubscriptionCursor) {
        let messages = if let Some(cryptor) = &client.cryptor {
            messages
                .into_iter()
                .map(|update| update.decrypt(cryptor))
                .collect()
        } else {
            messages
        };

        if let Some(manager) = client.subscription_manager().read().as_ref() {
            manager.notify_new_messages(cursor, messages.clone())
        }
    }

    /// Filter out `-pnpres` entries from the list.
    #[cfg(feature = "presence")]
    fn presence_filtered_entries(entries: Option<Vec<String>>) -> Option<Vec<String>> {
        entries.map(|channels| {
            channels
                .into_iter()
                .filter(|channel| !channel.ends_with("-pnpres"))
                .collect::<Vec<String>>()
        })
    }
}

impl<T, D> PubNubClientInstance<T, D> {
    /// Create subscription listener.
    ///
    /// Listeners configure [`PubNubClient`] to receive real-time updates for
    /// specified list of channels and groups.
    ///
    /// ```no_run // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// #   let client = PubNubClientBuilder::with_reqwest_transport()
    /// #      .with_keyset(Keyset {
    /// #          subscribe_key: "demo",
    /// #          publish_key: Some("demo"),
    /// #          secret_key: None,
    /// #      })
    /// #      .with_user_id("user_id")
    /// #      .build()?;
    /// client
    ///     .subscribe_raw()
    ///     .channels(["hello".into(), "world".into()].to_vec())
    ///     .execute()?
    ///     .stream()
    ///     .for_each(|update| async move {
    ///           println!("Received update: {:?}", update);
    ///     })
    ///     .await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    pub fn subscribe_raw(&self) -> RawSubscriptionBuilder<T, D> {
        RawSubscriptionBuilder {
            pubnub_client: Some(self.clone()),
            heartbeat: Some(self.config.presence.heartbeat_value),
            ..Default::default()
        }
    }

    /// Update real-time events filtering expression.
    ///
    /// # Arguments
    ///
    /// * `expression` - A `String` representing the filter expression.
    pub fn set_filter_expression<S>(&self, expression: S)
    where
        S: Into<String>,
    {
        let mut filter_expression = self.filter_expression.write();
        *filter_expression = expression.into();
    }

    /// Get real-time events filtering expression.
    ///
    /// # Returns
    ///
    /// Current real-time events filtering expression.
    pub fn get_filter_expression<S>(&self) -> Option<String> {
        let expression = self.filter_expression.read();
        (!expression.is_empty()).then(|| expression.clone())
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub(crate) fn subscribe_request(&self) -> SubscribeRequestBuilder<T, D> {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.clone()),
            heartbeat: Some(self.config.presence.heartbeat_value),
            ..Default::default()
        }
    }
}

// ===========================================================
// EventEmitter implementation for module PubNubClientInstance
// ===========================================================

#[cfg(feature = "std")]
impl<T, D> EventEmitter for PubNubClientInstance<T, D> {
    fn messages_stream(&self) -> DataStream<Message> {
        self.event_dispatcher.messages_stream()
    }

    fn signal_stream(&self) -> DataStream<Message> {
        self.event_dispatcher.signal_stream()
    }

    fn message_actions_stream(&self) -> DataStream<MessageAction> {
        self.event_dispatcher.message_actions_stream()
    }

    fn files_stream(&self) -> DataStream<File> {
        self.event_dispatcher.files_stream()
    }

    fn app_context_stream(&self) -> DataStream<AppContext> {
        self.event_dispatcher.app_context_stream()
    }

    fn presence_stream(&self) -> DataStream<Presence> {
        self.event_dispatcher.presence_stream()
    }

    fn stream(&self) -> DataStream<Update> {
        self.event_dispatcher.stream()
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod should {
    use futures::StreamExt;
    use spin::RwLock;

    use super::*;
    use crate::{
        core::{blocking, PubNubError, TransportRequest, TransportResponse},
        providers::deserialization_serde::DeserializerSerde,
        Keyset, PubNubClientBuilder, PubNubGenericClient,
    };

    #[derive(serde::Deserialize)]
    struct UserStateData {
        #[serde(rename = "admin")]
        pub is_admin: bool,
        #[serde(rename = "displayName")]
        pub display_name: String,
    }

    struct MockTransport {
        responses_count: RwLock<u16>,
    }

    impl Default for MockTransport {
        fn default() -> Self {
            Self {
                responses_count: RwLock::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
            let mut count_slot = self.responses_count.write();
            let response_body = generate_body(*count_slot);
            *count_slot += 1;

            if response_body.is_none() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }

            Ok(TransportResponse {
                status: 200,
                headers: [].into(),
                body: response_body,
            })
        }
    }

    impl blocking::Transport for MockTransport {
        fn send(&self, _req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            let mut count_slot = self.responses_count.write();
            let response_body = generate_body(*count_slot);
            *count_slot += 1;

            Ok(TransportResponse {
                status: 200,
                headers: [].into(),
                body: response_body,
            })
        }
    }

    fn generate_body(response_count: u16) -> Option<Vec<u8>> {
        match response_count {
            0 => Some(
                r#"{
                "t": {
                    "t": "15628652479902717",
                    "r": 4
                },
                "m": []
            }"#
                .into(),
            ),
            1 => Some(
                r#"{
                "t": {
                    "t": "15628652479932717",
                    "r": 4
                },
                "m": [
                    {
                        "a": "1",
                        "f": 514,
                        "i": "pn-0ca50551-4bc8-446e-8829-c70b704545fd",
                        "s": 1,
                        "p": {
                            "t": "15628652479933927",
                            "r": 4
                        },
                        "k": "demo",
                        "c": "my-channel",
                        "d": "my message",
                        "b": "my-channel"
                    },
                    {
                        "a": "5",
                        "f": 0,
                        "p": {
                            "r": 12,
                            "t": "15800701771129796"
                        },
                        "k": "demo",
                        "u": {
                            "pn_action": "state-change",
                            "pn_channel": "my-channel",
                            "pn_ispresence": 1,
                            "pn_occupancy": 1,
                            "pn_timestamp": 1580070177,
                            "pn_uuid": "pn-0ca50551-4bc8-446e-8829-c70b704545fd"
                        },
                        "c": "my-channel-pnpres",
                        "d": {
                            "action": "state-change",
                            "data": {
                                "admin": true,
                                "displayName": "ChannelAdmin"
                            },
                            "occupancy": 1,
                            "timestamp": 1580070177,
                            "uuid": "pn-0ca50551-4bc8-446e-8829-c70b704545fd"
                        },
                        "b": "my-channel-pnpres"
                    }
                ]
            }"#
                .into(),
            ),
            _ => None,
        }
    }

    fn client() -> PubNubGenericClient<MockTransport, DeserializerSerde> {
        PubNubClientBuilder::with_transport(Default::default())
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: None,
            })
            .with_user_id("user")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn create_subscription_set() {
        let _ = client().subscription(Some(&["channel_a"]), Some(&["group_a"]), None);
    }

    #[tokio::test]
    async fn subscribe() {
        let client = client();
        let subscription = client.subscription(
            Some(&["my-channel"]),
            Some(&["group_a"]),
            Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
        );
        subscription.subscribe(None);

        let status = client.status_stream().next().await.unwrap();
        let _ = subscription.messages_stream().next().await.unwrap();
        let presence = subscription.presence_stream().next().await.unwrap();

        assert!(matches!(status, ConnectionStatus::Connected));

        if let Presence::StateChange {
            timestamp: _,
            channel: _,
            subscription: _,
            uuid: _,
            data,
            ..
        } = presence
        {
            let user_data: UserStateData = serde_json::from_value(data)
                .expect("Should successfully deserialize user state object.");
            assert!(user_data.is_admin);
            assert_eq!(user_data.display_name, "ChannelAdmin");
        } else {
            panic!("Expected to receive presence update.")
        }

        client.unsubscribe_all();
    }

    #[tokio::test]
    async fn subscribe_raw() {
        let subscription = client()
            .subscribe_raw()
            .channels(["world".into()].to_vec())
            .execute()
            .unwrap();

        use futures::StreamExt;
        let message = subscription.stream().boxed().next().await;

        assert!(message.is_some());
    }

    #[test]
    fn subscribe_raw_blocking() {
        let subscription = client()
            .subscribe_raw()
            .channels(["world".into()].to_vec())
            .execute_blocking()
            .unwrap();

        let message = subscription.iter().next();

        assert!(message.is_some());
    }
}
