//! PubNub client module
//!
//! This module contains the [`PubNubClient`] struct.
//! It's used to send requests to [`PubNub API`].
//! It's intended to be used by the [`pubnub`] crate.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`pubnub`]: ../index.html

use derive_builder::Builder;
use log::info;
use spin::{Mutex, RwLock};
use uuid::Uuid;

#[cfg(all(
    any(feature = "subscribe", feature = "presence"),
    feature = "std",
    feature = "tokio"
))]
use crate::providers::futures_tokio::RuntimeTokio;
#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
use crate::subscribe::{EventDispatcher, SubscriptionCursor, SubscriptionManager};

#[cfg(all(feature = "presence", feature = "std"))]
use crate::presence::PresenceManager;

#[cfg(not(feature = "serde"))]
use crate::core::Deserializer;
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::DeserializerSerde;
#[cfg(feature = "reqwest")]
use crate::transport::TransportReqwest;

// TODO: Retry policy would be implemented for `no_std` event engine
#[cfg(feature = "std")]
use crate::{
    core::{retry_policy::Endpoint, runtime::RuntimeSupport, RequestRetryConfiguration},
    lib::alloc::vec,
};

use crate::{
    core::{CryptoProvider, PubNubEntity, PubNubError},
    lib::{
        alloc::{
            borrow::ToOwned,
            format,
            string::{String, ToString},
            sync::Arc,
            vec::Vec,
        },
        collections::HashMap,
        core::{
            cmp::max,
            ops::{Deref, DerefMut},
        },
    },
    transport::middleware::{PubNubMiddleware, SignatureKeySet},
    Channel, ChannelGroup, ChannelMetadata, UserMetadata,
};

/// PubNub client
///
/// Client for PubNub API with support for all [`selected`] PubNub features.
/// The client is transport-layer-agnostic, so you can use any transport layer
/// that implements the [`Transport`] trait.
///
/// You can create clients using the [`PubNubClient::with_transport`]
/// You must provide a valid [`Keyset`] with pub/sub keys and a string User ID
/// to identify the client.
///
/// To see available methods, please refer to the [`PubNubClientInstance`]
/// documentation.
///
/// # Examples
/// ```
/// use pubnub::{PubNubClientBuilder, Keyset};
///
/// // note that `with_reqwest_transport` requires `reqwest` feature
/// // to be enabled (default)
/// # fn main() -> Result<(), pubnub::core::PubNubError> {
/// let pubnub = PubNubClientBuilder::with_reqwest_transport()
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build()?;
///
/// # Ok(())
/// # }
/// ```
///
/// Using your own [`Transport`] implementation:
///
/// ```
/// use pubnub::{PubNubClientBuilder, Keyset};
///
/// # use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
/// # struct MyTransport;
/// # #[async_trait::async_trait]
/// # impl Transport for MyTransport {
/// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # impl MyTransport {
/// #     fn new() -> Self {
/// #         Self
/// #     }
/// # }
///
/// # fn main() -> Result<(), PubNubError> {
/// // note that MyTransport must implement the `Transport` trait
/// let transport = MyTransport::new();
///
/// let pubnub = PubNubClientBuilder::with_transport(MyTransport)
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build()?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Synchronization
///
/// Client is thread-safe and can be shared between threads. You don't need to
/// wrap it in `Arc` or `Mutex` because it is already wrapped in `Arc` and uses
/// interior mutability for its internal state.
///
/// # See also
/// [`Keyset`]
/// [`Transport`]
///
/// [`selected`]: ../index.html#features
/// [`Keyset`]: ../core/struct.Keyset.html
/// [`PubNubClient::with_transport`]: struct.PubNubClientBuilder.html#method.with_transport`
pub type PubNubGenericClient<T, D> = PubNubClientInstance<PubNubMiddleware<T>, D>;

/// PubNub client
///
/// Client for PubNub API with support for all [`selected`] PubNub features.
/// The client uses [`reqwest`] as a transport layer and [`serde`] for responses
/// deserialization.
///
/// You can create clients using the [`PubNubClient::with_reqwest_transport`]
/// method.
/// You must provide a valid [`Keyset`] with pub/sub keys and a string User ID
/// to identify the client.
///
/// To see available methods, please refer to the [`PubNubClientInstance`]
/// documentation.
///
/// # Examples
/// ```
/// use pubnub::{Keyset, PubNubClientBuilder};
///
/// // note that `with_reqwest_transport` requires `reqwest` feature
/// // to be enabled (default)
/// # fn main() -> Result<(), pubnub::core::PubNubError> {
/// let pubnub = PubNubClientBuilder::with_reqwest_transport()
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build()?;
///
/// # Ok(())
/// # }
/// ```
///
/// Using your own [`Transport`] implementation:
///
/// ```
/// # use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
/// use pubnub::{Keyset, PubNubClientBuilder};
///
/// # struct MyTransport;
/// # #[async_trait::async_trait]
/// # impl Transport for MyTransport {
/// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # impl MyTransport {
/// #     fn new() -> Self {
/// #         Self
/// #     }
/// # }
///
/// # fn main() -> Result<(), PubNubError> {
/// // note that MyTransport must implement the `Transport` trait
/// let transport = MyTransport::new();
///
/// let pubnub = PubNubClientBuilder::with_transport(MyTransport)
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build()?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Synchronization
///
/// Client is thread-safe and can be shared between threads. You don't need to
/// wrap it in `Arc` or `Mutex` because it is already wrapped in `Arc` and uses
/// interior mutability for its internal state.
///
/// # See also
/// [Keyset](struct.Keyset.html)
/// [Transport](../core/trait.Transport.html)
///
/// [`selected`]: ../index.html#features
/// [`Transport`]: ../core/trait.Transport.html
/// [`Keyset`]: ../core/struct.Keyset.html
/// [`reqwest`]: https://crates.io/crates/reqwest
/// [`PubNubClient::with_reqwest_transport`]: struct.PubNubClientBuilder.html#method.with_reqwest_transport
#[cfg(all(feature = "reqwest", feature = "serde"))]
pub type PubNubClient = PubNubGenericClient<TransportReqwest, DeserializerSerde>;

/// PubNub client raw instance.
///
/// This struct contains the actual client state.
/// It shouldn't be used directly. Use [`PubNubGenericClient`] or
/// [`PubNubClient`] instead.
#[derive(Debug)]
pub struct PubNubClientInstance<T, D> {
    pub(crate) inner: Arc<PubNubClientRef<T, D>>,

    /// Subscription time cursor.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) cursor: Arc<RwLock<Option<SubscriptionCursor>>>,

    /// Real-time event dispatcher.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) event_dispatcher: Arc<EventDispatcher>,
}

impl<T, D> Deref for PubNubClientInstance<T, D> {
    type Target = PubNubClientRef<T, D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, D> DerefMut for PubNubClientInstance<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to PubNubClientInstance are not allowed")
    }
}

impl<T, D> Clone for PubNubClientInstance<T, D> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),

            #[cfg(all(feature = "subscribe", feature = "std"))]
            cursor: Default::default(),

            #[cfg(all(feature = "subscribe", feature = "std"))]
            event_dispatcher: Arc::clone(&self.event_dispatcher),
        }
    }
}

/// Client reference
///
/// This struct contains the actual client state.
/// It's wrapped in `Arc` by [`PubNubClient`] and uses interior mutability for
/// its internal state.
///
/// Not intended to be used directly. Use [`PubNubClient`] instead.
#[derive(Builder, Debug)]
#[builder(
    pattern = "owned",
    name = "PubNubClientConfigBuilder",
    build_fn(private, name = "build_internal"),
    setter(prefix = "with"),
    no_std
)]
pub struct PubNubClientRef<T, D> {
    /// Transport layer
    pub(crate) transport: T,

    /// [`PubNub API`] responses deserializer
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub(crate) deserializer: Arc<D>,

    /// Data cryptor / decryptor provider
    #[builder(
        setter(custom, strip_option),
        field(vis = "pub(crate)"),
        default = "None"
    )]
    pub(crate) cryptor: Option<Arc<dyn CryptoProvider + Send + Sync>>,

    /// Instance ID
    #[builder(
        setter(into),
        field(ty = "String", build = "Arc::new(Some(Uuid::new_v4().to_string()))")
    )]
    pub(crate) instance_id: Arc<Option<String>>,

    /// Sequence number for the publish requests
    #[builder(default = "Mutex::new(1)")]
    pub(crate) next_seqn: Mutex<u16>,

    /// Configuration
    pub(crate) config: PubNubConfig,

    /// Access token
    #[builder(
        setter(custom),
        field(vis = "pub(crate)"),
        default = "Arc::new(spin::RwLock::new(String::new()))"
    )]
    pub(crate) auth_token: Arc<RwLock<String>>,

    /// Real-time data filtering expression.
    #[cfg(feature = "subscribe")]
    #[builder(
        setter(custom, strip_option),
        field(vis = "pub(crate)"),
        default = "Arc::new(spin::RwLock::new(String::new()))"
    )]
    pub(crate) filter_expression: Arc<RwLock<String>>,

    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// serialized `state` as values. State with heartbeat can be set **only**
    /// for channels.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # use pubnub::core::Serialize;
    /// #
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let state = HashMap::<String, Vec<u8>>::from([(
    ///     String::from("announce"),
    ///     HashMap::<String, bool>::from([
    ///         (String::from("is_owner"), false),
    ///         (String::from("is_admin"), true)
    ///     ]).serialize()?
    /// )]);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "presence")]
    #[builder(
        setter(custom),
        field(vis = "pub(crate)"),
        default = "Arc::new(spin::RwLock::new(HashMap::new()))"
    )]
    pub(crate) state: Arc<RwLock<HashMap<String, Vec<u8>>>>,

    /// Runtime environment
    #[cfg(feature = "std")]
    #[builder(setter(custom), field(vis = "pub(crate)"))]
    pub(crate) runtime: RuntimeSupport,

    /// Subscription module configuration
    ///
    /// > **Important**: Use `.subscription_manager()` to access it instead of
    /// > field.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) subscription: Arc<RwLock<Option<SubscriptionManager<T, D>>>>,

    /// Presence / heartbeat event engine.
    ///
    /// State machine which is responsible for `user_id` presence maintenance.
    ///
    /// > **Important**: Use `.presence_manager()` to access it instead of
    /// > field.
    #[cfg(all(feature = "presence", feature = "std"))]
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) presence: Arc<RwLock<Option<PresenceManager>>>,

    /// Created entities.
    ///
    /// Map of entities which has been created to access [`PubNub API`].
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) entities: RwLock<HashMap<String, PubNubEntity<T, D>>>,
}

impl<T, D> PubNubClientInstance<T, D> {
    /// Creates a new channel with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the channel as a string.
    ///
    /// # Returns
    ///
    /// Returns a `Channel` which can be used with the [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channel = pubnub.channel("my_channel");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channel<S>(&self, name: S) -> Channel<T, D>
    where
        S: Into<String>,
    {
        let mut entities_slot = self.entities.write();
        let name = name.into();
        let entity = entities_slot
            .entry(format!("{}_ch", &name))
            .or_insert(Channel::new(self, name).into());

        match entity {
            PubNubEntity::Channel(channel) => channel.clone(),
            _ => panic!("Unexpected entry type for Channel"),
        }
    }

    /// Creates a list of channels with the specified names.
    ///
    /// # Arguments
    ///
    /// * `names` - A list of names for the channels as a string.
    ///
    /// # Returns
    ///
    /// Returns a list of `Channel` which can be used with the [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channels = pubnub.channels(&["my_channel_1", "my_channel_2"]);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channels<S>(&self, names: &[S]) -> Vec<Channel<T, D>>
    where
        S: Into<String> + Clone,
    {
        let mut channels = Vec::with_capacity(names.len());
        let mut entities_slot = self.entities.write();

        for name in names.iter() {
            let name = name.to_owned().into();
            let entity = entities_slot
                .entry(format!("{}_ch", name))
                .or_insert(Channel::new(self, name).into());

            match entity {
                PubNubEntity::Channel(channel) => channels.push(channel.clone()),
                _ => panic!("Unexpected entry type for Channel"),
            }
        }

        channels
    }

    /// Creates a new channel group with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the channel group as a string.
    ///
    /// # Returns
    ///
    /// Returns a `ChannelGroup` which can be used with the [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channel_group = pubnub.channel_group("my_group");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channel_group<S>(&self, name: S) -> ChannelGroup<T, D>
    where
        S: Into<String>,
    {
        let mut entities_slot = self.entities.write();
        let name = name.into();
        let entity = entities_slot
            .entry(format!("{}_chg", &name))
            .or_insert(ChannelGroup::new(self, name).into());

        match entity {
            PubNubEntity::ChannelGroup(channel_group) => channel_group.clone(),
            _ => panic!("Unexpected entry type for ChannelGroup"),
        }
    }

    /// Creates a list of channel groups with the specified names.
    ///
    /// # Arguments
    ///
    /// * `name` - A list of names for the channel groups as a string.
    ///
    /// # Returns
    ///
    /// Returns a list of `ChannelGroup` which can be used with the [`PubNub
    /// API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channel_groups = pubnub.channel_groups(&["my_group_1", "my_group_2"]);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channel_groups<S>(&self, names: &[S]) -> Vec<ChannelGroup<T, D>>
    where
        S: Into<String> + Clone,
    {
        let mut channel_groups = Vec::with_capacity(names.len());
        let mut entities_slot = self.entities.write();

        for name in names.iter() {
            let name = name.clone().into();
            let entity = entities_slot
                .entry(format!("{}_chg", name))
                .or_insert(ChannelGroup::new(self, name).into());

            match entity {
                PubNubEntity::ChannelGroup(channel_group) => {
                    channel_groups.push(channel_group.clone())
                }
                _ => panic!("Unexpected entry type for ChannelGroup"),
            }
        }

        channel_groups
    }

    /// Creates a new channel metadata object with the specified identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the channel metadata object as a string.
    ///
    /// # Returns
    ///
    /// Returns a `ChannelMetadata` which can be used with the [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channel_metadata = pubnub.channel_metadata("channel_meta");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channel_metadata<S>(&self, id: S) -> ChannelMetadata<T, D>
    where
        S: Into<String>,
    {
        let mut entities_slot = self.entities.write();
        let id = id.into();
        let entity = entities_slot
            .entry(format!("{}_chm", &id))
            .or_insert(ChannelMetadata::new(self, id).into());

        match entity {
            PubNubEntity::ChannelMetadata(channel_metadata) => channel_metadata.clone(),
            _ => panic!("Unexpected entry type for ChannelMetadata"),
        }
    }

    /// Creates a list of channel metadata objects with the specified
    /// identifiers.
    ///
    /// # Arguments
    ///
    /// * `id` - A list of identifiers for the channel metadata objects as a
    ///   string.
    ///
    /// # Returns
    ///
    /// Returns a list of `ChannelMetadata` which can be used with the [`PubNub
    /// API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let channels_metadata = pubnub.channels_metadata(
    ///     &["channel_meta_1", "channel_meta_2"]
    /// );
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn channels_metadata<S>(&self, ids: &[S]) -> Vec<ChannelMetadata<T, D>>
    where
        S: Into<String> + Clone,
    {
        let mut channels_metadata = Vec::with_capacity(ids.len());
        let mut entities_slot = self.entities.write();

        for id in ids.iter() {
            let id = id.clone().into();
            let entity = entities_slot
                .entry(format!("{}_chm", id))
                .or_insert(ChannelMetadata::new(self, id).into());

            match entity {
                PubNubEntity::ChannelMetadata(channel_metadata) => {
                    channels_metadata.push(channel_metadata.clone())
                }
                _ => panic!("Unexpected entry type for ChannelMetadata"),
            }
        }

        channels_metadata
    }

    /// Creates a new user metadata object with the specified identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the user metadata object as a string.
    ///
    /// # Returns
    ///
    /// Returns a `UserMetadata` which can be used with the [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let user_metadata = pubnub.user_metadata("user_meta");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn user_metadata<S>(&self, id: S) -> UserMetadata<T, D>
    where
        S: Into<String>,
    {
        let mut entities_slot = self.entities.write();
        let id = id.into();
        let entity = entities_slot
            .entry(format!("{}_uidm", &id))
            .or_insert(UserMetadata::new(self, id).into());

        match entity {
            PubNubEntity::UserMetadata(user_metadata) => user_metadata.clone(),
            _ => panic!("Unexpected entry type for UserMetadata"),
        }
    }

    /// Creates a list of user metadata objects with the specified identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - A list of identifiers for the user metadata objects as a
    ///   string.
    ///
    /// # Returns
    ///
    /// Returns a list of `UserMetadata` which can be used with the
    /// [`PubNub API`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// let users_metadata = pubnub.users_metadata(&["user_meta_1", "user_meta_2"]);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub fn users_metadata<S>(&self, ids: &[S]) -> Vec<UserMetadata<T, D>>
    where
        S: Into<String> + Clone,
    {
        let mut users_metadata = Vec::with_capacity(ids.len());
        let mut entities_slot = self.entities.write();

        for id in ids.iter() {
            let id = id.clone().into();
            let entity = entities_slot
                .entry(format!("{}_uidm", id))
                .or_insert(UserMetadata::new(self, id).into());

            match entity {
                PubNubEntity::UserMetadata(user_metadata) => {
                    users_metadata.push(user_metadata.clone())
                }
                _ => panic!("Unexpected entry type for UserMetadata"),
            }
        }

        users_metadata
    }

    /// Update currently used authentication token.
    ///
    /// # Examples
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let token = "<auth token from grant_token>";
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub.set_token(token);
    /// // Now client has access to all endpoints for which `token` has
    /// // permissions.
    /// #     Ok(())
    /// # }
    /// ```
    pub fn set_token<S>(&self, access_token: S)
    where
        S: Into<String>,
    {
        let mut token = self.auth_token.write();
        *token = access_token.into();
    }

    /// Retrieve currently used authentication token.
    ///
    /// # Examples
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// #     let token = "<auth token from grant_token>";
    /// let pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// #     pubnub.set_token(token);
    /// println!("Current authentication token: {:?}", pubnub.get_token());
    /// // Now client has access to all endpoints for which `token` has
    /// // permissions.
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_token(&self) -> Option<String> {
        let token = self.auth_token.read().deref().clone();
        (!token.is_empty()).then_some(token)
    }
}

impl<T, D> PubNubClientInstance<T, D>
where
    T: crate::core::Transport + Send + Sync + 'static,
    D: crate::core::Deserializer + Send + Sync + 'static,
{
    /// Terminates the subscription and presence managers if the corresponding
    /// features are enabled.
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub fn terminate(&self) {
        #[cfg(feature = "subscribe")]
        {
            let manager = self.subscription_manager(false);
            let mut manager_slot = manager.write();
            if let Some(manager) = manager_slot.as_ref() {
                manager.terminate();
            }
            // Free up resources used by subscription event engine.
            *manager_slot = None;
        }
        #[cfg(feature = "presence")]
        {
            let manager = self.presence_manager(false);
            let mut manager_slot = manager.write();
            if let Some(manager) = manager_slot.as_ref() {
                manager.terminate();
            }
            *manager_slot = None;
        }
    }
}

impl<T, D> PubNubClientConfigBuilder<T, D> {
    /// Set client authentication key.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    pub fn with_auth_key<S>(mut self, auth_key: S) -> Self
    where
        S: Into<String>,
    {
        if let Some(configuration) = self.config.as_mut() {
            configuration.auth_key = Some(Arc::new(auth_key.into()));
        }

        self
    }

    /// `user_id` presence heartbeat.
    ///
    /// Used to set the presence timeout period. It overrides the default value
    /// of 300 seconds for Presence timeout.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub fn with_heartbeat_value(mut self, value: u64) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            let value = max(20, value);
            configuration.presence.heartbeat_value = value;

            #[cfg(feature = "std")]
            {
                configuration.presence.heartbeat_interval = Some(value / 2 - 1);
            }
        }
        self
    }

    /// `user_id` presence announcement interval.
    ///
    /// Intervals at which `user_id` presence should be announced and should
    /// follow this optimal formula: `heartbeat_value / 2 - 1`.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub fn with_heartbeat_interval(mut self, interval: u64) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.presence.heartbeat_interval = Some(interval);
        }
        self
    }

    /// Whether `user_id` leave should be announced or not.
    ///
    /// When set to `true` and `user_id` will unsubscribe, the client wouldn't
    /// announce `leave`, and as a result, there will be no `leave` presence
    /// event generated.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub fn with_suppress_leave_events(mut self, suppress_leave_events: bool) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.presence.suppress_leave_events = suppress_leave_events;
        }
        self
    }

    /// Requests automatic retry configuration.
    ///
    /// The retry configuration regulates the frequency of request retry
    /// attempts and the number of failed attempts that should be retried.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    #[cfg(feature = "std")]
    pub fn with_retry_configuration(
        mut self,
        retry_configuration: RequestRetryConfiguration,
    ) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.transport.retry_configuration = retry_configuration;
        }

        self
    }

    /// Data encryption / decryption
    ///
    /// Crypto module used by client when publish messages / signals and receive
    /// them as real-time updates from subscription module.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    pub fn with_cryptor<C>(mut self, cryptor: C) -> Self
    where
        C: CryptoProvider + Send + Sync + 'static,
    {
        self.cryptor = Some(Some(Arc::new(cryptor)));

        self
    }

    /// Real-time events filtering expression.
    ///
    /// # Arguments
    ///
    /// * `expression` - A `String` representing the filter expression.
    ///
    /// # Returns
    ///
    /// [`PubNubClientConfigBuilder`] that you can use to set the configuration
    /// for the client. This is a part of the [`PubNubClientConfigBuilder`].
    #[cfg(feature = "subscribe")]
    pub fn with_filter_expression<S>(self, expression: S) -> Self
    where
        S: Into<String>,
    {
        if let Some(filter_expression) = &self.filter_expression {
            let mut filter_expression = filter_expression.write();
            *filter_expression = expression.into();
        }

        self
    }

    /// Build a [`PubNubClient`] from the builder
    pub fn build(self) -> Result<PubNubClientInstance<PubNubMiddleware<T>, D>, PubNubError> {
        self.build_internal()
            .map_err(|err| PubNubError::ClientInitialization {
                details: err.to_string(),
            })
            .and_then(|pre_build| {
                let token = Arc::new(RwLock::new(String::new()));
                #[cfg(all(feature = "subscribe", feature = "std"))]
                let subscription = Arc::new(RwLock::new(None));
                #[cfg(all(feature = "presence", feature = "std"))]
                let presence: Arc<RwLock<Option<PresenceManager>>> = Arc::new(RwLock::new(None));

                info!(
                    "Client Configuration: \n publish_key: {:?}\n subscribe_key: {}\n user_id: {}\n instance_id: {:?}",
                    pre_build.config.publish_key,
                    pre_build.config.subscribe_key,
                    pre_build.config.user_id,
                    pre_build.instance_id
                );

                Ok(PubNubClientRef {
                    transport: PubNubMiddleware {
                        signature_keys: pre_build.config.clone().signature_key_set()?,
                        auth_key: pre_build.config.auth_key.clone(),
                        instance_id: pre_build.instance_id.clone(),
                        user_id: pre_build.config.user_id.clone(),
                        transport: pre_build.transport,
                        auth_token: token.clone(),
                    },
                    deserializer: pre_build.deserializer,
                    instance_id: pre_build.instance_id,
                    next_seqn: pre_build.next_seqn,
                    auth_token: token,
                    config: pre_build.config,
                    cryptor: pre_build.cryptor.clone(),

                    #[cfg(feature = "subscribe")]
                    filter_expression: pre_build.filter_expression,

                    #[cfg(feature = "presence")]
                    state: Arc::new(RwLock::new(HashMap::new())),

                    #[cfg(feature = "std")]
                    runtime: pre_build.runtime,

                    #[cfg(all(feature = "subscribe", feature = "std"))]
                    subscription: subscription.clone(),

                    #[cfg(all(feature = "presence", feature = "std"))]
                    presence: presence.clone(),

                    entities: RwLock::new(HashMap::new()),
                })
            })
            .map(|client| {
                PubNubClientInstance {
                    inner: Arc::new(client),

                    #[cfg(all(feature = "subscribe", feature = "std"))]
                    cursor: Default::default(),

                    #[cfg(all(feature = "subscribe", feature = "std"))]
                    event_dispatcher: Default::default(),
                }
            })
    }
}

/// Transport specific configuration
///
/// Configuration let specify timeouts for two types of requests:
/// * `subscribe` - long-poll requests
/// * `non-subscribe` - any non-subscribe requests.
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportConfiguration {
    /// Timeout after which subscribe request will be cancelled by timeout.
    pub subscribe_request_timeout: u64,

    /// Timeout after which any non-subscribe request will be cancelled by
    /// timeout.
    pub request_timeout: u64,

    /// Request automatic retry configuration.
    ///
    /// Automatic retry configuration contains a retry policy that should be
    /// used to calculate retry delays and the number of attempts that
    /// should be made.
    pub(crate) retry_configuration: RequestRetryConfiguration,
}

#[cfg(feature = "std")]
impl Default for TransportConfiguration {
    fn default() -> Self {
        Self {
            subscribe_request_timeout: 310,
            request_timeout: 10,
            retry_configuration: RequestRetryConfiguration::Exponential {
                min_delay: 2,
                max_delay: 150,
                max_retry: 6,
                excluded_endpoints: Some(vec![
                    Endpoint::MessageSend,
                    Endpoint::Presence,
                    Endpoint::Files,
                    Endpoint::MessageStorage,
                    Endpoint::ChannelGroups,
                    Endpoint::DevicePushNotifications,
                    Endpoint::AppContext,
                    Endpoint::MessageReactions,
                ]),
            },
        }
    }
}

/// `user_id` presence behaviour configuration.
///
/// The configuration contains parameters to control when the timeout may occur
/// or whether any updates should be sent when leaving.
#[cfg(any(feature = "subscribe", feature = "presence"))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PresenceConfiguration {
    /// `user_id` presence heartbeat.
    ///
    /// Used to set the presence timeout period. It overrides the default value
    /// of 300 seconds for Presence timeout.
    pub heartbeat_value: u64,

    /// `user_id` presence announcement interval.
    ///
    /// Intervals at which `user_id` presence should be announced and should
    /// follow this optimal formula: `heartbeat_value / 2 - 1`.
    #[cfg(feature = "std")]
    pub heartbeat_interval: Option<u64>,

    /// Whether `user_id` leave should be announced or not.
    ///
    /// When set to `true` and `user_id` will unsubscribe, the client wouldn't
    /// announce `leave`, and as a result, there will be no `leave` presence
    /// event generated.
    ///
    /// **Default:** `false`
    pub suppress_leave_events: bool,
}

#[cfg(any(feature = "subscribe", feature = "presence"))]
impl Default for PresenceConfiguration {
    fn default() -> Self {
        Self {
            heartbeat_value: 300,
            suppress_leave_events: false,

            #[cfg(feature = "std")]
            heartbeat_interval: None,
        }
    }
}

/// PubNub configuration
///
/// Configuration for [`PubNubClient`].
/// This struct separates the configuration from the actual client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubNubConfig {
    /// Subscribe key
    pub(crate) subscribe_key: String,

    /// Publish key
    pub(crate) publish_key: Option<String>,

    /// Secret key
    pub(crate) secret_key: Option<String>,

    /// User ID
    pub(crate) user_id: Arc<String>,

    /// Authorization key
    pub(crate) auth_key: Option<Arc<String>>,

    /// Transport configuration.
    ///
    /// Configuration allow to configure request processing aspects like:
    /// * timeout
    /// * automatic retry.
    #[cfg(feature = "std")]
    pub transport: TransportConfiguration,

    /// Presence configuration.
    ///
    /// The configuration allows you to set up `user_id` channels presence:
    /// * the period after which an `user_id` _timeout_ event will occur
    /// * how often the server should be notified about user presence
    /// * whether `user_id` _leave_ event should be announced or not.
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub presence: PresenceConfiguration,
}

impl PubNubConfig {
    fn signature_key_set(self) -> Result<Option<SignatureKeySet>, PubNubError> {
        if let Some(secret_key) = self.secret_key {
            #[cfg(not(feature = "std"))]
            log::warn!("Signature calculation is not supported in `no_std`` environment!");

            let publish_key = self.publish_key.ok_or(PubNubError::ClientInitialization {
                details: "You must also provide the publish key if you use the secret key."
                    .to_string(),
            })?;
            Ok(Some(SignatureKeySet {
                secret_key,
                publish_key,
                subscribe_key: self.subscribe_key,
            }))
        } else {
            Ok(None)
        }
    }
}

/// PubNub builder for [`PubNubClient`]
///
/// Builder for [`PubNubClient`] that is a first step to create a client.
/// The client is transport-layer-agnostic, so you can use any transport layer
/// that implements the [`Transport`] trait.
///
/// You can use the [`Default`] implementation to create a builder
/// with the default transport layer, responses deserializer and runtime
/// environment (if `tokio` feature enabled). The `default` method is
/// implemented only when the `reqwest` and `serde` features is enabled.
///
/// The builder provides methods to set the transport layer and returns the next
/// step of the builder with the remaining parameters.
///
/// See [`PubNubClient`] for more information.
#[derive(Debug, Clone)]
pub struct PubNubClientBuilder;

impl PubNubClientBuilder {
    /// Set the transport layer for the client.
    ///
    /// Returns [`PubNubClientRuntimeBuilder`] where depending on from enabled
    /// `features` following can be set:
    /// * runtime environment
    /// * API ket set to access [`PubNub API`].
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl Transport for MyTransport {
    /// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let pubnub = PubNubClientBuilder::with_transport(transport)
    ///     .with_keyset(Keyset {
    ///         publish_key: Some("pub-c-abc123"),
    ///         subscribe_key: "sub-c-abc123",
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub fn with_transport<T>(transport: T) -> PubNubClientRuntimeBuilder<T>
    where
        T: crate::core::Transport,
    {
        PubNubClientRuntimeBuilder { transport }
    }

    /// Set the transport layer for the client.
    ///
    /// Returns [`PubNubClientDeserializerBuilder`] where depending on from
    /// enabled `features` following can be set:
    /// * [`PubNub API`] response deserializer
    /// * API ket set to access [`PubNub API`].
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl Transport for MyTransport {
    /// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let pubnub = PubNubClientBuilder::with_transport(transport)
    ///     .with_keyset(Keyset {
    ///         publish_key: Some("pub-c-abc123"),
    ///         subscribe_key: "sub-c-abc123",
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(any(
        all(not(feature = "subscribe"), not(feature = "presence")),
        not(feature = "std")
    ))]
    pub fn with_transport<T>(transport: T) -> PubNubClientDeserializerBuilder<T> {
        PubNubClientDeserializerBuilder { transport }
    }

    /// Set the blocking transport layer for the client.
    ///
    /// Returns [`PubNubClientRuntimeBuilder`] where depending on from enabled
    /// `features` following can be set:
    /// * runtime environment
    /// * API ket set to access [`PubNub API`].
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::{blocking::Transport, TransportRequest, TransportResponse, PubNubError};
    /// use pubnub::{Keyset, PubNubClientBuilder};
    ///
    /// # struct MyTransport;
    /// # impl Transport for MyTransport {
    /// #     fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let pubnub = PubNubClientBuilder::with_blocking_transport(transport)
    ///     .with_keyset(Keyset {
    ///         publish_key: Some("pub-c-abc123"),
    ///         subscribe_key: "sub-c-abc123",
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(
        any(feature = "subscribe", feature = "presence"),
        feature = "std",
        feature = "blocking"
    ))]
    pub fn with_blocking_transport<T>(transport: T) -> PubNubClientRuntimeBuilder<T>
    where
        T: crate::core::blocking::Transport + Send + Sync,
    {
        PubNubClientRuntimeBuilder { transport }
    }

    /// Set the blocking transport layer for the client.
    ///
    /// Returns [`PubNubClientDeserializerBuilder`] where depending on from
    /// enabled `features` following can be set:
    /// * [`PubNub API`] response deserializer
    /// * API ket set to access [`PubNub API`].
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::{blocking::Transport, TransportRequest, TransportResponse, PubNubError};
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// # struct MyTransport;
    /// # impl Transport for MyTransport {
    /// #     fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let pubnub = PubNubClientBuilder::with_blocking_transport(transport)
    ///     .with_keyset(Keyset {
    ///         publish_key: Some("pub-c-abc123"),
    ///         subscribe_key: "sub-c-abc123",
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(feature = "blocking")]
    #[cfg(any(
        all(not(feature = "subscribe"), not(feature = "presence")),
        not(feature = "std")
    ))]
    pub fn with_blocking_transport<T>(transport: T) -> PubNubClientDeserializerBuilder<T>
    where
        T: crate::core::blocking::Transport + Send + Sync,
    {
        PubNubClientDeserializerBuilder { transport }
    }
}

/// PubNub builder for [`PubNubClient`] to set API keys.
///
/// The builder provides methods to set the [`PubNub API`] keys set and returns
/// the next step of the builder with the remaining parameters.
///
/// See [`PubNubClient`] for more information.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[derive(Debug, Clone)]
pub struct PubNubClientKeySetBuilder<T, D> {
    /// Transport layer.
    pub(crate) transport: T,

    /// [`PubNub API`] responses deserializer
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub(crate) deserializer: D,

    /// Runtime environment
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub(crate) runtime: RuntimeSupport,
}

impl<T, D> PubNubClientKeySetBuilder<T, D> {
    /// Set the keyset for the client
    ///
    /// It returns [`PubNubClientUserIdBuilder`] builder that you can use
    /// to set the User ID for the client.
    ///
    /// See [`Keyset`] for more information.
    ///
    /// # Examples
    /// ```
    /// use pubnub::{Keyset, PubNubClientBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let builder = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientUserIdBuilder`]: struct.PubNubClientUserIdBuilder.html
    /// [`Keyset`]: struct.Keyset.html
    pub fn with_keyset<S>(self, keyset: Keyset<S>) -> PubNubClientUserIdBuilder<T, S, D>
    where
        S: Into<String>,
    {
        PubNubClientUserIdBuilder {
            transport: self.transport,
            deserializer: self.deserializer,
            keyset,
            #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
            runtime: self.runtime,
        }
    }
}

/// PubNub builder for [`PubNubClient`] used to set runtime environment.
///
/// Runtime will be used for detached tasks spawning and delayed task execution.
///
/// Depending on from enabled `features` methods may return:
/// * [`PubNubClientDeserializerBuilder`] to set custom [`PubNub API`]
///   deserializer
/// * [`PubNubClientKeySetBuilder`] to set API keys set to access [`PubNub API`]
/// * [`PubNubClientUserIdBuilder`] to set user id for the client.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
pub struct PubNubClientRuntimeBuilder<T> {
    /// Transport layer.
    pub(crate) transport: T,
}

#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
impl<T> PubNubClientRuntimeBuilder<T> {
    /// Set runtime environment.
    ///
    /// Returns [`PubNubClientDeserializerBuilder`] where depending on from
    /// enabled `features` following can be set:
    /// * [`PubNub API`] response deserializer
    /// * API ket set to access [`PubNub API`].
    ///
    /// See [`Runtime`] trait for more information.
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::Runtime;
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::future::Future;
    /// #
    /// # #[derive(Clone)]
    /// # struct MyRuntime;
    /// #
    /// # #[async_trait::async_trait]
    /// # impl Runtime for MyRuntime {
    /// #     fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
    /// #         // spawn the Future
    /// #         // e.g. tokio::spawn(future);
    /// #     }
    /// #
    /// #     async fn sleep(self, _delay: u64) {
    /// #         // e.g. tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await
    /// #     }
    /// #
    /// #     async fn sleep_microseconds(self, delay: u64) {
    /// #         // e.g. tokio::time::sleep(tokio::time::Duration::from_micros(delay)).await
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let pubnub = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_runtime(MyRuntime)
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("user-123")
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientDeserializerBuilder`]: struct.PubNubClientDeserializerBuilder.html
    /// [`Runtime`]: trait.Runtime.html
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(not(feature = "serde"), not(feature = "tokio")))]
    pub fn with_runtime<R>(self, runtime: R) -> PubNubClientDeserializerBuilder<T>
    where
        R: Runtime + Send + Sync + 'static,
    {
        PubNubClientDeserializerBuilder {
            transport: self.transport,
            runtime: RuntimeSupport::new(Arc::new(runtime)),
        }
    }

    /// Set runtime environment.
    ///
    /// It returns [`PubNubClientKeySetBuilder`] builder that you can use
    /// to set API ket set to access [`PubNub API`].
    ///
    /// See [`Runtime`] trait for more information.
    ///
    /// # Examples
    /// ```
    /// # use pubnub::core::Runtime;
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::future::Future;
    /// #
    /// # #[derive(Clone)]
    /// # struct MyRuntime;
    /// #
    /// # #[async_trait::async_trait]
    /// # impl Runtime for MyRuntime {
    /// #     fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
    /// #         // spawn the Future
    /// #         // e.g. tokio::spawn(future);
    /// #     }
    /// #
    /// #     async fn sleep(self, _delay: u64) {
    /// #         // e.g. tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await
    /// #     }
    /// #
    /// #     async fn sleep_microseconds(self, delay: u64) {
    /// #         // e.g. tokio::time::sleep(tokio::time::Duration::from_micros(delay)).await
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let pubnub = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_runtime(MyRuntime)
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("user-123")
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientKeySetBuilder`]: struct.PubNubClientKeySetBuilder.html
    /// [`Runtime`]: trait.Runtime.html
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(feature = "serde", not(feature = "tokio")))]
    pub fn with_runtime<R>(self, runtime: R) -> PubNubClientKeySetBuilder<T, DeserializerSerde>
    where
        R: Runtime + Send + Sync + 'static,
    {
        PubNubClientKeySetBuilder {
            transport: self.transport,
            deserializer: DeserializerSerde,
            runtime: RuntimeSupport::new(Arc::new(runtime)),
        }
    }

    /// Set the keyset for the client.
    ///
    /// It returns [`PubNubClientUserIdBuilder`] builder that you can use
    /// to set User ID for the client.
    ///
    /// See [`Keyset`] for more information.
    ///
    /// # Examples
    /// ```
    /// use pubnub::{Keyset, PubNubClientBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let builder = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientUserIdBuilder`]: struct.PubNubClientUserIdBuilder.html
    /// [`Keyset`]: struct.Keyset.html
    #[cfg(all(feature = "serde", feature = "tokio"))]
    pub fn with_keyset<S>(
        self,
        keyset: Keyset<S>,
    ) -> PubNubClientUserIdBuilder<T, S, DeserializerSerde>
    where
        S: Into<String>,
    {
        PubNubClientUserIdBuilder {
            transport: self.transport,
            deserializer: DeserializerSerde,
            keyset,
            runtime: RuntimeSupport::new(Arc::new(RuntimeTokio)),
        }
    }
}

/// PubNub builder for [`PubNubClient`] used to set custom deserializer.
///
/// Deserializer will be used to process request responses from [`PubNub API`].
///
/// See [`PubNubClientDeserializerBuilder`] for more information.
///
/// [`reqwest`]: https://docs.rs/reqwest
/// [`PubNub API`]: https://www.pubnub.com/docs
pub struct PubNubClientDeserializerBuilder<T> {
    /// Transport later.
    pub(crate) transport: T,

    /// Runtime environment
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub(crate) runtime: RuntimeSupport,
}

impl<T> PubNubClientDeserializerBuilder<T> {
    /// Set [`PubNub API`] responses deserializer.
    ///
    /// It returns [`PubNubClientKeySetBuilder`] builder that you can use
    /// to set API ket set to access [`PubNub API`].
    ///
    /// See [`Deserializer`] for more information.
    ///
    ///
    /// # Examples
    /// ```
    /// # use pubnub::{
    /// #     core::{Deserializer, PubNubError},
    /// #     dx::presence::{result::HeartbeatResponseBody, HeartbeatResult},
    /// # };
    /// use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// # struct MyDeserializer;
    /// #
    /// # impl Deserializer for MyDeserializer {
    /// #     fn deserialize<HeartbeatResponseBody>(
    /// #         &self, response: &[u8]
    /// #     ) -> Result<HeartbeatResult, PubNubError> {
    /// #         // ...
    /// #         Ok(HeartbeatResult)
    /// #     }
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let builder = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_deserializer(MyDeserializer)
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientRuntimeBuilder`]: struct.PubNubClientRuntimeBuilder.html
    /// [`Deserializer`]: trait.Deserializer.html
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(not(feature = "serde"))]
    pub fn with_deserializer<D>(self, deserializer: D) -> PubNubClientKeySetBuilder<T, D>
    where
        D: Deserializer,
    {
        PubNubClientKeySetBuilder {
            transport: self.transport,
            deserializer,
            #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
            runtime: self.runtime,
        }
    }

    /// Set the keyset for the client.
    ///
    /// It returns [`PubNubClientUserIdBuilder`] builder that you can use
    /// to set User ID for the client.
    ///
    /// See [`Keyset`] for more information.
    ///
    /// # Examples
    /// ```
    /// use pubnub::{Keyset, PubNubClientBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let builder = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PubNubClientUserIdBuilder`]: struct.PubNubClientUserIdBuilder.html
    /// [`Keyset`]: struct.Keyset.html
    #[cfg(feature = "serde")]
    pub fn with_keyset<S>(
        self,
        keyset: Keyset<S>,
    ) -> PubNubClientUserIdBuilder<T, S, DeserializerSerde>
    where
        S: Into<String>,
    {
        PubNubClientUserIdBuilder {
            transport: self.transport,
            deserializer: DeserializerSerde,
            keyset,

            #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
            runtime: self.runtime,
        }
    }
}

/// PubNub builder for [`PubNubClient`] used to set the User ID
/// It is returned by [`PubNubClientBuilder::with_keyset`]
/// and provides method to set the User ID for the client.
///
/// See [`PubNubClient`] for more information.
///
/// # Examples
/// ```
/// use pubnub::{PubNubClientBuilder, Keyset};
///
/// let builder = PubNubClientBuilder::with_reqwest_transport()
/// .with_keyset(Keyset {
///     subscribe_key: "sub-c-abc123",
///     publish_key: Some("pub-c-abc123"),
///     secret_key: None,
/// })
/// .with_user_id("my-user_id");
/// ```
///
/// # See also
/// [`Keyset`]
/// [`PubNubClientBuilder`]
/// [`PubNubClient`]
///
/// [`PubNubClientBuilder::with_keyset`]: struct.PubNubClientBuilder.html#method.with_keyset
#[derive(Debug, Clone)]
pub struct PubNubClientUserIdBuilder<T, S, D>
where
    S: Into<String>,
{
    transport: T,
    deserializer: D,
    keyset: Keyset<S>,

    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    runtime: RuntimeSupport,
}

impl<T, S, D> PubNubClientUserIdBuilder<T, S, D>
where
    S: Into<String>,
{
    /// Set user id for the client.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use
    /// to set the configuration for the client. This is a part
    /// the PubNubClientConfigBuilder.
    ///
    /// [`PubNubClientConfigBuilder`]: struct.PubNubClientConfigBuilder.html
    pub fn with_user_id<U>(self, user_id: U) -> PubNubClientConfigBuilder<T, D>
    where
        U: Into<String>,
    {
        let publish_key = self.keyset.publish_key.map(|k| k.into());
        let secret_key = self.keyset.secret_key.map(|k| k.into());

        PubNubClientConfigBuilder {
            transport: Some(self.transport),
            config: Some(PubNubConfig {
                publish_key,
                subscribe_key: self.keyset.subscribe_key.into(),
                secret_key,
                user_id: Arc::new(user_id.into()),
                auth_key: None,

                #[cfg(feature = "std")]
                transport: Default::default(),

                #[cfg(any(feature = "subscribe", feature = "presence"))]
                presence: Default::default(),
            }),

            #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
            runtime: Some(self.runtime),

            deserializer: Some(Arc::new(self.deserializer)),
            ..Default::default()
        }
    }
}

/// Keyset for the PubNub client
///
/// # Examples
/// ```
/// use pubnub::Keyset;
///
/// Keyset {
///    subscribe_key: "sub-c-abc123",
///    publish_key: Some("pub-c-abc123"),
///    secret_key: Some("sec-c-abc123"),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Keyset<S>
where
    S: Into<String>,
{
    /// Subscribe key
    pub subscribe_key: S,

    /// Publish key
    pub publish_key: Option<S>,

    /// Secret key
    pub secret_key: Option<S>,
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::{TransportRequest, TransportResponse};
    use std::any::type_name;

    #[test]
    fn include_pubnub_middleware() {
        #[derive(Default)]
        struct MockTransport;

        #[async_trait::async_trait]
        impl crate::core::Transport for MockTransport {
            async fn send(
                &self,
                _request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                Ok(TransportResponse::default())
            }
        }

        fn type_of<T>(_: &T) -> &'static str {
            type_name::<T>()
        }

        let client = PubNubClientBuilder::with_transport(MockTransport)
            .with_keyset(Keyset {
                subscribe_key: "",
                publish_key: Some(""),
                secret_key: None,
            })
            .with_user_id("my-user_id")
            .build()
            .unwrap();

        assert_eq!(
            type_of(&client.transport),
            type_name::<PubNubMiddleware<MockTransport>>()
        );
    }

    #[test]
    fn publish_key_is_required_if_secret_is_set() {
        let config = PubNubConfig {
            publish_key: None,
            subscribe_key: "sub_key".into(),
            secret_key: Some("sec_key".into()),
            user_id: Arc::new("".into()),
            auth_key: None,

            #[cfg(feature = "std")]
            transport: Default::default(),

            #[cfg(any(feature = "subscribe", feature = "presence"))]
            presence: Default::default(),
        };

        assert!(config.signature_key_set().is_err());
    }
}
