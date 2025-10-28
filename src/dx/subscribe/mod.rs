//! Subscribe module.
//!
//! Allows to subscribe to real-time updates from channels and groups.

#[cfg(feature = "std")]
use futures::{future::BoxFuture, FutureExt};
#[cfg(feature = "std")]
use spin::RwLock;

#[cfg(feature = "std")]
use crate::{
    core::{Deserializer, PubNubError, Transport},
    lib::alloc::{boxed::Box, sync::Arc, vec::Vec},
    subscribe::result::SubscribeResult,
};

#[cfg(feature = "std")]
use crate::core::{
    event_engine::{CancellationTask, EventEngine},
    runtime::Runtime,
    DataStream, PubNubEntity,
};

use crate::{
    dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String,
    subscribe::raw::RawSubscriptionBuilder,
};

#[cfg(all(feature = "presence", feature = "std"))]
use event_engine::SubscriptionInput;
#[cfg(feature = "std")]
use event_engine::{SubscribeEffectHandler, SubscribeEventEngine, SubscribeState};

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
impl<T, D> PubNubClientInstance<T, D>
where
    T: Transport + Send + 'static,
    D: Deserializer + Send + 'static,
{
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
        self.event_dispatcher.handle_status(status.clone());
        let mut should_terminate = false;

        {
            if let Some(manager) = self.subscription_manager(false).read().as_ref() {
                should_terminate = !manager.has_handlers();
            }
        }

        // Terminate event engine because there is no event listeners (registered
        // Subscription and SubscriptionSet instances).
        if matches!(status, ConnectionStatus::Disconnected) && should_terminate {
            self.terminate()
        }
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
    /// let pubnub = // PubNubClient
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
    /// let empty_pubnub_client = pubnub.clone_empty();
    /// // self.other_component(empty_pubnub_client);    
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
    D: Deserializer + Send + 'static,
{
    /// Creates multiplexed subscriptions.
    ///
    /// # Arguments
    ///
    /// * `parameters` - [`SubscriptionParams`] configuration object.
    ///
    /// # Returns
    ///
    /// The created [`SubscriptionSet`] object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use futures::StreamExt;
    /// use pubnub::{
    ///     subscribe::{
    ///         EventEmitter, {EventSubscriber, SubscriptionParams},
    ///     },
    ///     Keyset, PubNubClient, PubNubClientBuilder,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let subscription = pubnub.subscription(SubscriptionParams {
    ///     channels: Some(&["my_channel_1", "my_channel_2", "my_channel_3"]),
    ///     channel_groups: None,
    ///     options: None
    /// });
    /// // Message stream for handling real-time `Message` events.
    /// let stream = subscription.messages_stream();
    /// #     Ok(())
    /// # }
    /// ```
    pub fn subscription<N>(&self, parameters: SubscriptionParams<N>) -> SubscriptionSet<T, D>
    where
        N: Into<String> + Clone,
    {
        let mut entities: Vec<PubNubEntity<T, D>> = vec![];
        if let Some(channel_names) = parameters.channels {
            entities.extend(
                channel_names
                    .iter()
                    .cloned()
                    .map(|name| self.channel(name).into())
                    .collect::<Vec<PubNubEntity<T, D>>>(),
            );
        }
        if let Some(channel_group_names) = parameters.channel_groups {
            entities.extend(
                channel_group_names
                    .iter()
                    .cloned()
                    .map(|name| self.channel_group(name).into())
                    .collect::<Vec<PubNubEntity<T, D>>>(),
            );
        }

        SubscriptionSet::new(entities, parameters.options)
    }

    /// Stop receiving real-time updates.
    ///
    /// Stop receiving real-time updates for previously subscribed channels and
    /// groups by temporarily disconnecting from the [`PubNub`] network.
    ///
    /// ```no_run
    /// use futures::StreamExt;
    /// use pubnub::{
    ///     subscribe::{
    ///         EventEmitter, {EventSubscriber, SubscriptionParams, Update},
    ///     },
    ///     Keyset, PubNubClient, PubNubClientBuilder,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let pubnub = PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #             subscribe_key: "demo",
    /// #             publish_key: Some("demo"),
    /// #             secret_key: None,
    /// #         })
    /// #         .with_user_id("user_id")
    /// #         .build()?;
    /// # let subscription = pubnub.subscription(SubscriptionParams {
    /// #     channels: Some(&["channel"]),
    /// #     channel_groups: None,
    /// #     options: None
    /// # });
    /// # let stream = // DataStream<Message>
    /// #     subscription.messages_stream();
    /// pubnub.disconnect();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub`]: https://www.pubnub.com
    pub fn disconnect(&self) {
        #[cfg(feature = "presence")]
        let mut input: Option<SubscriptionInput> = None;

        if let Some(manager) = self.subscription_manager(false).read().as_ref() {
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
            } else if let Some(presence) = self.presence_manager(false).read().as_ref() {
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
    /// use pubnub::{
    ///     subscribe::{
    ///         EventEmitter, {EventSubscriber, SubscriptionParams, Update},
    ///     },
    ///     Keyset, PubNubClient, PubNubClientBuilder,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let pubnub = PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #             subscribe_key: "demo",
    /// #             publish_key: Some("demo"),
    /// #             secret_key: None,
    /// #         })
    /// #         .with_user_id("user_id")
    /// #         .build()?;
    /// # let subscription = pubnub.subscription(SubscriptionParams {
    /// #     channels: Some(&["channel"]),
    /// #     channel_groups: None,
    /// #     options: None
    /// # });
    /// # let stream = // DataStream<Message>
    /// #     subscription.messages_stream();
    /// # // .....
    /// # pubnub.disconnect();
    /// pubnub.reconnect(None);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub`]: https://www.pubnub.com
    pub fn reconnect(&self, cursor: Option<SubscriptionCursor>) {
        #[cfg(feature = "presence")]
        let mut input: Option<SubscriptionInput> = None;
        let cursor = cursor.or_else(|| self.cursor.read().clone());

        if let Some(manager) = self.subscription_manager(false).read().as_ref() {
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
            } else if let Some(presence) = self.presence_manager(false).read().as_ref() {
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
            let mut cursor = self.cursor.write();
            *cursor = None;

            if let Some(manager) = self.subscription_manager(false).write().as_mut() {
                manager.unregister_all()
            }
        }
    }

    /// Subscription manager which maintains Subscription EE.
    ///
    /// # Arguments
    ///
    /// `create` - Whether manager should be created if not initialized.
    ///
    /// # Returns
    ///
    /// Returns an [`SubscriptionManager`] which represents the manager.
    #[cfg(feature = "subscribe")]
    pub(crate) fn subscription_manager(
        &self,
        create: bool,
    ) -> Arc<RwLock<Option<SubscriptionManager<T, D>>>> {
        {
            let manager = self.subscription.read();
            if manager.is_some() || !create {
                return self.subscription.clone();
            }
        }

        {
            // Initialize subscription module when it will be first required.
            let mut slot = self.subscription.write();
            if slot.is_none() && create {
                #[cfg(feature = "presence")]
                let heartbeat_self = self.clone();
                #[cfg(feature = "presence")]
                let leave_self = self.clone();

                *slot = Some(SubscriptionManager::new(
                    self.subscribe_event_engine(),
                    #[cfg(feature = "presence")]
                    Arc::new(move |channels, groups, _all| {
                        Self::subscribe_heartbeat_call(heartbeat_self.clone(), channels, groups);
                    }),
                    #[cfg(feature = "presence")]
                    Arc::new(move |channels, groups, all| {
                        Self::subscribe_leave_call(leave_self.clone(), channels, groups, all);
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
        let runtime = self.runtime.clone();
        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);

        EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |params| {
                    Self::subscribe_call(
                        subscribe_client.clone(),
                        params.clone(),
                        cancel_rx.clone(),
                    )
                }),
                Arc::new(move |status| Self::emit_status(emit_status_client.clone(), &status)),
                Arc::new(Box::new(move |updates, cursor: SubscriptionCursor| {
                    Self::emit_messages(emit_messages_client.clone(), updates, cursor)
                })),
                cancel_tx,
            ),
            SubscribeState::Unsubscribed,
            runtime,
        )
    }

    fn subscribe_call(
        client: Self,
        params: event_engine::types::SubscriptionParams,
        cancel_rx: async_channel::Receiver<String>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
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

        request.execute_with_cancel(cancel_task).boxed()
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
        client.announce_join(
            Self::presence_filtered_entries(channels),
            Self::presence_filtered_entries(channel_groups),
        );
    }

    /// Subscription event engine presence `leave` announcement.
    ///
    /// The leave call method provides few different flows based on the
    /// presence event engine state:
    /// * can operate - call `leave` announcement
    /// * can't operate (heartbeat interval not set) - make direct `leave` call.
    #[cfg(all(feature = "presence", feature = "std"))]
    fn subscribe_leave_call(
        client: Self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
        all: bool,
    ) {
        if !all {
            client.announce_left(
                Self::presence_filtered_entries(channels),
                Self::presence_filtered_entries(channel_groups),
            );
        } else {
            client.announce_left_all()
        }
    }

    fn emit_status(client: Self, status: &ConnectionStatus) {
        if let Some(manager) = client.subscription_manager(false).read().as_ref() {
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

        if let Some(manager) = client.subscription_manager(false).read().as_ref() {
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
    /// ```no_run
    /// // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::{
    ///     subscribe::{
    ///         EventEmitter, {EventSubscriber, SubscriptionParams, Update},
    ///     },
    ///     Keyset, PubNubClient, PubNubClientBuilder,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #   let pubnub = PubNubClientBuilder::with_reqwest_transport()
    /// #      .with_keyset(Keyset {
    /// #          subscribe_key: "demo",
    /// #          publish_key: Some("demo"),
    /// #          secret_key: None,
    /// #      })
    /// #      .with_user_id("user_id")
    /// #      .build()?;
    /// pubnub
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

    fn signals_stream(&self) -> DataStream<Message> {
        self.event_dispatcher.signals_stream()
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

    /// Requests handler function type.
    type RequestHandler = Box<dyn Fn(&TransportRequest) + Send + Sync>;

    #[derive(Default)]
    struct MockTransportWithHandler {
        ///  Response which mocked transport should return.
        response: Option<TransportResponse>,

        /// Request handler function which will be called before returning
        /// response.
        ///
        /// Use function to verify request parameters.
        request_handler: Option<RequestHandler>,
    }

    #[async_trait::async_trait]
    impl Transport for MockTransportWithHandler {
        async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            // Calling request handler (if provided).
            if let Some(handler) = &self.request_handler {
                handler(&req);
            }

            Ok(self.response.clone().unwrap_or(transport_response(200)))
        }
    }

    /// Service response payload.
    fn transport_response(status: u16) -> TransportResponse {
        TransportResponse {
            status,
            body: Some(Vec::from(if status < 400 {
                "{\"t\":{\"t\":\"17613449864766754\",\"r\":21},\"m\":[]}"
            } else {
                "\"error\":{{\"message\":\"Overall error\",\"source\":\"test\",\"details\":[{{\"message\":\"Error\",\"location\":\"signature\",\"locationType\":\"query\"}}]}}"
            })),
            ..Default::default()
        }
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

    fn client<T>(transport: Option<T>) -> PubNubGenericClient<T, DeserializerSerde>
    where
        T: Transport + Default,
    {
        PubNubClientBuilder::with_transport(transport.unwrap_or_default())
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
        let _ = client::<MockTransport>(None).subscription(SubscriptionParams {
            channels: Some(&["channel_a"]),
            channel_groups: Some(&["group_a"]),
            options: None,
        });
    }

    #[tokio::test]
    async fn subscribe() {
        let client = client::<MockTransport>(None);
        let subscription = client.subscription(SubscriptionParams {
            channels: Some(&["my-channel"]),
            channel_groups: Some(&["group_a"]),
            options: Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
        });
        subscription.subscribe();

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
    async fn subscribe_with_unique_channels_and_groups() {
        let transport = MockTransportWithHandler {
            response: None,
            request_handler: Some(Box::new(|req| {
                if req.path.starts_with("/v2/subscribe") {
                    assert_eq!(req.path.split('/').collect::<Vec<&str>>()[4], "channel_a");
                    assert_eq!(req.query_parameters["channel-group"], "group_b");
                }
            })),
        };
        let client = client(Some(transport));
        let subscription = client.subscription(SubscriptionParams {
            channels: Some(&["channel_a", "channel_a", "channel_a"]),
            channel_groups: Some(&["group_b", "group_b", "group_b"]),
            options: None,
        });
        subscription.subscribe();

        let status = client.status_stream().next().await.unwrap();

        assert!(matches!(status, ConnectionStatus::Connected));
    }

    #[tokio::test]
    async fn subscribe_raw() {
        let subscription = client::<MockTransport>(None)
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
        let subscription = client::<MockTransport>(None)
            .subscribe_raw()
            .channels(["world".into()].to_vec())
            .execute_blocking()
            .unwrap();

        let message = subscription.iter().next();

        assert!(message.is_some());
    }
}
