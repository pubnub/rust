//! # Subscription module.
//!
//! This module contains the [`Subscription`] type, which is used to manage
//! subscription to the specific entity and attach listeners to process
//! real-time events triggered for the `entity`.

use spin::RwLock;
use uuid::Uuid;

use crate::core::{Deserializer, Transport};
use crate::{
    core::{DataStream, PubNubEntity},
    dx::pubnub_client::PubNubClientInstance,
    lib::{
        alloc::{
            string::String,
            sync::{Arc, Weak},
            vec,
            vec::Vec,
        },
        collections::HashMap,
        core::{
            cmp::PartialEq,
            fmt::{Debug, Formatter, Result},
            ops::{Deref, DerefMut},
        },
    },
    subscribe::{
        event_engine::SubscriptionInput, traits::EventHandler, AppContext, EventDispatcher,
        EventEmitter, EventSubscriber, File, Message, MessageAction, Presence, SubscribableType,
        SubscriptionCursor, SubscriptionOptions, SubscriptionSet, Update,
    },
};

/// Entity subscription.
///
/// # Example
///
/// ### Multiplexed subscription
///
/// ```rust
/// use pubnub::{
///     subscribe::{Subscriber, SubscriptionOptions}, Keyset, PubNubClient, PubNubClientBuilder,
/// };
///
/// # fn main() -> Result<(), pubnub::core::PubNubError> {
/// let client = // PubNubClient
/// #     PubNubClientBuilder::with_reqwest_transport()
/// #         .with_keyset(Keyset {
/// #              subscribe_key: "demo",
/// #              publish_key: Some("demo"),
/// #              secret_key: Some("demo")
/// #          })
/// #         .with_user_id("uuid")
/// #         .build()?;
/// let channel = client.create_channel("my_channel");
/// let subscription = channel.subscription(None);
/// // Subscription with presence announcements
/// let subscription_with_presence = channel.subscription(Some(vec![SubscriptionOptions::ReceivePresenceEvents]));
/// #     Ok(())
/// # }
/// ```
///
/// ### Sum of subscriptions
///
/// ```rust
/// use pubnub::{
///     subscribe::{Subscriber, Subscription, SubscriptionSet},
///     Keyset, PubNubClient, PubNubClientBuilder,
/// };
///
/// # fn main() -> Result<(), pubnub::core::PubNubError> {
/// let client = // PubNubClient
/// #     PubNubClientBuilder::with_reqwest_transport()
/// #         .with_keyset(Keyset {
/// #              subscribe_key: "demo",
/// #              publish_key: Some("demo"),
/// #              secret_key: Some("demo")
/// #          })
/// #         .with_user_id("uuid")
/// #         .build()?;
/// let channels = client.create_channels(&["my_channel_1", "my_channel_2"]);
/// let subscription = channels[0].subscription(None).add(channels[1].subscription(None));
/// #     Ok(())
/// # }
/// ```
pub struct Subscription<T: Send + Sync, D: Send + Sync> {
    /// Unique event handler instance identifier.
    ///
    /// [`Subscription`] can be cloned, but the internal state is always bound
    /// to the same reference of [`SubscriptionRef`] with the same `id`.
    instance_id: String,

    /// Subscription reference.
    inner: Arc<SubscriptionRef<T, D>>,

    /// Real-time event dispatcher.
    event_dispatcher: EventDispatcher,
}

/// Subscription reference
///
/// This struct contains the actual subscription state.
/// It's wrapped in `Arc` by [`Subscription`] and uses interior mutability for
/// its internal state.
///
/// Not intended to be used directly. Use [`Subscription`] instead.
#[derive(Debug)]
pub struct SubscriptionRef<T: Send + Sync, D: Send + Sync> {
    /// Unique event handler identifier.
    pub(super) id: String,

    /// [`PubNubClientInstance`] which is backing which subscription.
    pub(super) client: Weak<PubNubClientInstance<T, D>>,

    /// Subscribable entity.
    ///
    /// Entity with information that is required to receive real-time updates
    /// for it.
    pub(super) entity: PubNubEntity<T, D>,

    /// Whether set is currently subscribed and active.
    pub(super) is_subscribed: Arc<RwLock<bool>>,

    /// List of strings which represent data stream identifiers for entity
    /// real-time events.
    pub(super) subscription_input: SubscriptionInput,

    /// Subscription time cursor.
    cursor: RwLock<Option<SubscriptionCursor>>,

    /// Subscription listener options.
    ///
    /// Options used to set up listener behavior and real-time events
    /// processing.
    options: Option<Vec<SubscriptionOptions>>,

    /// The list of weak references to all [`Subscription`] clones created for
    /// this reference.
    clones: RwLock<HashMap<String, Weak<Subscription<T, D>>>>,
}

impl<T, D> Subscription<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    /// Creates a new subscription for specified entity.
    ///
    /// # Arguments
    ///
    /// * `client` - Weak reference on [`PubNubClientInstance`] to access shared
    ///   resources.
    /// * `entities` - A `PubNubEntity` representing the entity to subscribe to.
    /// * `options` - An optional list of `SubscriptionOptions` specifying the
    ///   subscription behaviour.
    ///
    /// # Returns
    ///
    /// A new `Subscription2` for the given `entity` and `options`.
    pub(crate) fn new(
        client: Weak<PubNubClientInstance<T, D>>,
        entity: PubNubEntity<T, D>,
        options: Option<Vec<SubscriptionOptions>>,
    ) -> Arc<Self> {
        let subscription_ref = SubscriptionRef::new(client, entity, options);
        let subscription_id = Uuid::new_v4().to_string();
        let subscription = Arc::new(Self {
            instance_id: subscription_id.clone(),
            inner: Arc::new(subscription_ref),
            event_dispatcher: Default::default(),
        });
        subscription.store_clone(subscription_id, Arc::downgrade(&subscription));
        subscription
    }

    /// Retrieves the current timetoken value.
    ///
    /// # Returns
    ///
    /// The current timetoken value as an `usize`, or 0 if the timetoken cannot
    /// be parsed.
    pub(super) fn current_timetoken(&self) -> usize {
        self.cursor
            .read()
            .as_ref()
            .and_then(|cursor| cursor.timetoken.parse::<usize>().ok())
            .unwrap_or(0)
    }

    /// Checks if the [`Subscription`] is active or not.
    ///
    /// # Returns
    ///
    /// Returns `true` if the active, otherwise `false`.
    pub(super) fn is_subscribed(&self) -> bool {
        *self.is_subscribed.read()
    }

    /// Clones the [`Subscription`] and returns a new `Arc` reference to it.
    ///
    /// # Returns
    ///
    /// A new `Arc` reference to a cloned [`Subscription` ] instance.
    ///
    /// # Panics
    ///
    /// This method will panic if [`Subscription`] clone could not be found in
    /// the reference counter storage or if there are no strong references
    /// to the [`Subscription`] instance.
    pub fn clone_arc(&self) -> Arc<Self> {
        self.get_clone_by_id(&self.instance_id)
            .expect("Subscription clone should be stored with SubscriptionRef")
            .upgrade()
            .expect("At least one strong reference should exist for Subscription")
            .clone()
    }

    /// Creates a clone of the subscription set with an empty event dispatcher.
    ///
    /// Empty clones have the same subscription state but an empty list of
    /// real-time event listeners, which makes it possible to attach
    /// listeners specific to the context. When the cloned subscription goes out
    /// of scope, all associated listeners will be invalidated and released.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{subscribe::Subscriber, PubNubClient, PubNubClientBuilder, Keyset};
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
    /// let channel = client.create_channel("my_channel");
    /// let subscription = channel.subscription(None);
    /// // ...
    /// // We need to pass subscription into other component which would like to
    /// // have own listeners to handle real-time events.
    /// let empty_subscription = subscription.clone_empty();
    /// // self.other_component(empty_subscription);    
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// A new instance of the subscription object with an empty event
    /// dispatcher.
    pub fn clone_empty(&self) -> Arc<Self> {
        let instance_id = Uuid::new_v4().to_string();
        let instance = Arc::new(Self {
            instance_id: instance_id.clone(),
            inner: Arc::clone(&self.inner),
            event_dispatcher: Default::default(),
        });
        self.store_clone(instance_id, Arc::downgrade(&instance));
        instance
    }

    /// Adds two [`Subscription`] and produce [`SubscriptionSet`].
    ///
    /// # Arguments
    ///
    /// * `rhs` - The subscription to be added.
    ///
    /// # Returns
    ///
    /// [`SubscriptionSet`] with added subscriptions in the set.
    ///
    /// # Panics
    ///
    /// This function will panic if the current subscription set does not have
    /// at least one clone of [`Subscription`] with strong reference to the
    /// original.
    pub fn add(&self, rhs: Arc<Self>) -> Arc<SubscriptionSet<T, D>> {
        let options = self.options.clone();
        let lhs_clones = self.clones.read();
        let (_, lhs) = lhs_clones
            .iter()
            .next()
            .expect("At least one clone of Subscription should exist.");
        let lhs = lhs
            .upgrade()
            .clone()
            .expect("At least one strong reference should exist for Subscription");
        SubscriptionSet::new_with_subscriptions(vec![lhs, rhs], options)
    }

    /// Filters the given list of `Update` events based on the subscription
    /// input and the current timetoken.
    ///
    /// # Arguments
    ///
    /// * `events` - A slice of `Update` events to filter.
    ///
    /// # Returns
    ///
    /// A new `Vec<Update>` containing only the events that satisfy the
    /// following conditions:
    /// 1. The event's subscription is present in the subscription input.
    /// 2. The event's timestamp is greater than or equal to the current
    ///    timetoken.
    fn filtered_events(&self, events: &[Update]) -> Vec<Update> {
        let subscription_input = self.subscription_input(true);
        let current_timetoken = self.current_timetoken();

        events
            .iter()
            .filter(|event| {
                subscription_input.contains(&event.subscription())
                    && event.event_timestamp().ge(&current_timetoken)
            })
            .cloned()
            .collect::<Vec<Update>>()
    }
}

impl<T, D> Deref for Subscription<T, D>
where
    T: Send + Sync,
    D: Send + Sync,
{
    type Target = SubscriptionRef<T, D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, D> DerefMut for Subscription<T, D>
where
    T: Send + Sync,
    D: Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to the Subscription are not allowed")
    }
}

impl<T, D> PartialEq for Subscription<T, D>
where
    T: Send + Sync,
    D: Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<T, D> Debug for Subscription<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "Subscription {{ id: {}, instance_id: {}, entity: {:?}, subscription_input: {:?}, \
            is_subscribed: {}, cursor: {:?}, options: {:?}}}",
            self.id,
            self.instance_id,
            self.entity,
            self.subscription_input,
            self.is_subscribed(),
            self.cursor.read().clone(),
            self.options
        )
    }
}

impl<T, D> EventSubscriber for Subscription<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn subscribe(&self, cursor: Option<SubscriptionCursor>) {
        let mut is_subscribed = self.is_subscribed.write();
        if *is_subscribed {
            return;
        }
        *is_subscribed = true;

        if cursor.is_some() {
            let mut cursor_slot = self.cursor.write();
            if let Some(current_cursor) = cursor_slot.as_ref() {
                let catchup_cursor = cursor.clone().unwrap_or_default();
                catchup_cursor
                    .gt(current_cursor)
                    .then(|| *cursor_slot = Some(catchup_cursor));
            } else {
                *cursor_slot = cursor.clone();
            }
        }

        if let Some(client) = self.client().upgrade().clone() {
            if let Some(manager) = client.subscription_manager().write().as_mut() {
                // Mark entities as "in use" by subscription.
                self.entity.increase_subscriptions_count();

                if let Some((_, handler)) = self.clones.read().iter().next() {
                    let handler: Weak<dyn EventHandler<T, D> + Send + Sync> = handler.clone();
                    manager.register(&handler, cursor);
                }
            }
        }
    }

    fn unsubscribe(&self) {
        {
            let mut is_subscribed_slot = self.is_subscribed.write();
            if !*is_subscribed_slot {
                return;
            }
            *is_subscribed_slot = false;
        }

        if let Some(client) = self.client().upgrade().clone() {
            if let Some(manager) = client.subscription_manager().write().as_mut() {
                // Mark entities as "not in-use" by subscription.
                self.entity.decrease_subscriptions_count();

                if let Some((_, handler)) = self.clones.read().iter().next() {
                    let handler: Weak<dyn EventHandler<T, D> + Send + Sync> = handler.clone();
                    manager.unregister(&handler);
                }
            }
        }
    }
}

impl<T, D> EventHandler<T, D> for Subscription<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn handle_events(&self, cursor: SubscriptionCursor, events: &[Update]) {
        if !self.is_subscribed() {
            return;
        }

        let filtered_events = self.filtered_events(events);

        let mut cursor_slot = self.cursor.write();
        if let Some(current_cursor) = cursor_slot.as_ref() {
            cursor
                .gt(current_cursor)
                .then(|| *cursor_slot = Some(cursor));
        } else {
            *cursor_slot = Some(cursor);
        }

        // Go through subscription clones and trigger events for them.
        self.clones.write().retain(|_, handler| {
            if let Some(handler) = handler.upgrade().clone() {
                handler
                    .event_dispatcher
                    .handle_events(filtered_events.clone());
                return true;
            }
            false
        });
    }

    fn subscription_input(&self, include_inactive: bool) -> SubscriptionInput {
        if !include_inactive && self.entity.subscriptions_count().eq(&0) {
            Default::default()
        }

        self.subscription_input.clone()
    }

    fn invalidate(&self) {
        {
            let mut is_subscribed = self.is_subscribed.write();
            if !*is_subscribed {
                return;
            }
            *is_subscribed = false;
        }
        self.entity.decrease_subscriptions_count();

        // Go through subscription clones and invalidate them.
        self.clones.write().retain(|_, handler| {
            if let Some(handler) = handler.upgrade().clone() {
                handler.event_dispatcher.invalidate();
                return true;
            }
            false
        });
    }

    fn id(&self) -> &String {
        &self.id
    }

    fn client(&self) -> Weak<PubNubClientInstance<T, D>> {
        self.client.clone()
    }
}

impl<T, D> EventEmitter for Subscription<T, D>
where
    T: Send + Sync,
    D: Send + Sync,
{
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

impl<T, D> SubscriptionRef<T, D>
where
    T: Send + Sync,
    D: Send + Sync,
{
    fn new(
        client: Weak<PubNubClientInstance<T, D>>,
        entity: PubNubEntity<T, D>,
        options: Option<Vec<SubscriptionOptions>>,
    ) -> SubscriptionRef<T, D> {
        let is_channel_type = matches!(entity.r#type(), SubscribableType::Channel);
        let with_presence = if let Some(options) = &options {
            options
                .iter()
                .any(|option| matches!(option, SubscriptionOptions::ReceivePresenceEvents))
        } else {
            false
        };
        let entity_names = entity.names(with_presence);

        let input = SubscriptionInput::new(
            &is_channel_type.then(|| entity_names.clone()),
            &(!is_channel_type).then_some(entity_names),
        );

        Self {
            id: Uuid::new_v4().to_string(),
            client,
            entity,
            is_subscribed: Default::default(),
            subscription_input: input,
            cursor: Default::default(),
            options,
            clones: Default::default(),
        }
    }

    /// Store a clone of a [`Subscription`] instance with a given instance ID.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The instance ID to associate with the clone.
    /// * `instance` - The weak reference to the subscription instance to store
    ///   as a clone.
    fn store_clone(&self, instance_id: String, instance: Weak<Subscription<T, D>>) {
        let mut clones = self.clones.write();
        (!clones.contains_key(&instance_id)).then(|| clones.insert(instance_id, instance));
    }

    /// Retrieves a cloned instance of a [`Subscription`] by its `instance_id`.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - A reference to the unique identifier of the instance.
    ///
    /// # Returns
    ///
    /// An `Option` containing a weak reference to the cloned [`Subscription`]
    /// instance if found, or `None` if no instance with the specified
    /// `instance_id` exists.
    fn get_clone_by_id(&self, instance_id: &String) -> Option<Weak<Subscription<T, D>>> {
        self.clones.read().get(instance_id).cloned()
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{Channel, ChannelGroup, Keyset, PubNubClient, PubNubClientBuilder};

    fn client() -> PubNubClient {
        PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: None,
            })
            .with_user_id("user")
            .build()
            .unwrap()
    }

    #[test]
    fn create_subscription_from_channel_entity() {
        let client = Arc::new(client());
        let channel = Channel::new(&client, "channel");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::Channel(channel),
            None,
        );

        assert!(!subscription.is_subscribed());
        assert!(subscription.subscription_input.contains_channel("channel"));
    }

    #[test]
    fn create_subscription_from_channel_entity_with_options() {
        let client = Arc::new(client());
        let channel = Channel::new(&client, "channel");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::Channel(channel),
            Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
        );

        assert!(!subscription.is_subscribed());
        assert!(subscription.subscription_input.contains_channel("channel"));
        assert!(subscription
            .subscription_input
            .contains_channel("channel-pnpres"));
    }

    #[test]
    fn create_subscription_from_channel_group_entity() {
        let client = Arc::new(client());
        let channel_group = ChannelGroup::new(&client, "channel-group");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::ChannelGroup(channel_group),
            None,
        );

        assert!(!subscription.is_subscribed());
        assert!(subscription
            .subscription_input
            .contains_channel_group("channel-group"));
        assert!(!subscription
            .subscription_input
            .contains_channel_group("channel-group-pnpres"));
    }

    #[test]
    fn create_subscription_from_channel_group_entity_with_options() {
        let client = Arc::new(client());
        let channel_group = ChannelGroup::new(&client, "channel-group");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::ChannelGroup(channel_group),
            Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
        );

        assert!(!subscription.is_subscribed());
        assert!(subscription
            .subscription_input
            .contains_channel_group("channel-group"));
        assert!(subscription
            .subscription_input
            .contains_channel_group("channel-group-pnpres"));
    }

    #[test]
    fn preserve_id_between_clones() {
        let client = Arc::new(client());
        let channel = Channel::new(&client, "channel");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::Channel(channel),
            None,
        );
        assert_eq!(subscription.clone().id.clone(), subscription.id.clone());
    }

    #[test]
    fn preserve_options_between_clones() {
        let client = Arc::new(client());
        let channel = Channel::new(&client, "channel");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::Channel(channel),
            Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
        );
        assert_eq!(
            subscription.clone().options.clone(),
            subscription.options.clone()
        );
    }

    #[test]
    fn not_preserve_listeners_between_clones() {
        let client = Arc::new(client());
        let channel = Channel::new(&client, "channel");
        let subscription = Subscription::new(
            Arc::downgrade(&client),
            PubNubEntity::Channel(channel),
            None,
        );
        let _ = subscription.messages_stream();

        assert_eq!(
            subscription
                .event_dispatcher
                .message_streams
                .read()
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        assert!(subscription
            .clone_empty()
            .event_dispatcher
            .message_streams
            .read()
            .as_ref()
            .is_none());
    }
}
