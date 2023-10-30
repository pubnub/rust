//! PubNub client module
//!
//! This module contains the [`PubNubClient`] struct.
//! It's used to send requests to [`PubNub API`].
//! It's intended to be used by the [`pubnub`] crate.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`pubnub`]: ../index.html

#[cfg(all(feature = "presence", feature = "std"))]
use crate::presence::PresenceManager;
#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
use crate::{
    core::runtime::RuntimeSupport, providers::futures_tokio::RuntimeTokio,
    subscribe::SubscriptionManager,
};

#[cfg(not(feature = "serde"))]
use crate::core::Deserializer;
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::DeserializerSerde;
#[cfg(feature = "reqwest")]
use crate::transport::TransportReqwest;

// TODO: Retry policy would be implemented for `no_std` event engine
#[cfg(feature = "std")]
use crate::core::RequestRetryPolicy;

use crate::{
    core::{CryptoProvider, PubNubError},
    lib::{
        alloc::{
            string::{String, ToString},
            sync::Arc,
        },
        core::ops::{Deref, DerefMut},
    },
    transport::middleware::{PubNubMiddleware, SignatureKeySet},
};
use derive_builder::Builder;
use log::info;
use spin::{Mutex, RwLock};
use uuid::Uuid;

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
/// let client = PubNubClientBuilder::with_reqwest_transport()
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
/// let client = PubNubClientBuilder::with_transport(MyTransport)
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
/// [`PubNubClient::with_transport`]: struct.PubNubClientBuilder.html#method.with_transport`]
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
/// let client = PubNubClientBuilder::with_reqwest_transport()
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
/// let client = PubNubClientBuilder::with_transport(MyTransport)
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
        field(type = "String", build = "Arc::new(Some(Uuid::new_v4().to_string()))")
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

    /// Runtime environment
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    #[builder(setter(custom), field(vis = "pub(crate)"))]
    pub(crate) runtime: RuntimeSupport,

    /// Subscription module configuration
    #[cfg(all(feature = "subscribe", feature = "std"))]
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) subscription: Arc<RwLock<Option<SubscriptionManager>>>,

    /// Presence / heartbeat event engine.
    ///
    /// State machine which is responsible for `user_id` presence maintenance.
    #[cfg(all(feature = "presence", feature = "std"))]
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) presence: Arc<RwLock<Option<PresenceManager>>>,
}

impl<T, D> PubNubClientInstance<T, D> {
    /// Update currently used authentication token.
    ///
    /// # Examples
    /// ```rust
    /// use pubnub::{PubNubClient, PubNubClientBuilder, Keyset};
    ///
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let token = "<auth token from grant_token>";
    /// let client = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// client.set_token(token);
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
    /// let client = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// #     client.set_token(token);
    /// println!("Current authentication token: {:?}", client.get_token());
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
            configuration.heartbeat_value = value;
            configuration.heartbeat_interval = Some(value / 2 - 1);
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
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub fn with_heartbeat_interval(mut self, interval: u64) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.heartbeat_interval = Some(interval);
        }
        self
    }

    /// Requests retry policy.
    ///
    /// The retry policy regulates the frequency of request retry attempts and
    /// the number of failed attempts that should be retried.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part of the
    /// [`PubNubClientConfigBuilder`].
    #[cfg(feature = "std")]
    pub fn with_retry_policy(mut self, policy: RequestRetryPolicy) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.retry_policy = policy;
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

    /// Build a [`PubNubClient`] from the builder
    pub fn build(self) -> Result<PubNubClientInstance<PubNubMiddleware<T>, D>, PubNubError> {
        self.build_internal()
            .map_err(|err| PubNubError::ClientInitialization {
                details: err.to_string(),
            })
            .and_then(|pre_build| {
                let token = Arc::new(RwLock::new(String::new()));
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

                    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
                    runtime: pre_build.runtime,

                    #[cfg(all(feature = "subscribe", feature = "std"))]
                    subscription: Arc::new(RwLock::new(None)),

                    #[cfg(all(feature = "presence", feature = "std"))]
                    presence: Arc::new(RwLock::new(None)),
                })
            })
            .map(|client| PubNubClientInstance {
                inner: Arc::new(client),
            })
    }
}

/// PubNub configuration
///
/// Configuration for [`PubNubClient`].
/// This struct separates the configuration from the actual client.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Request retry policy
    #[cfg(feature = "std")]
    pub(crate) retry_policy: RequestRetryPolicy,

    /// `user_id` presence heartbeat.
    ///
    /// Used to set the presence timeout period. It overrides the default value
    /// of 300 seconds for Presence timeout.
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub heartbeat_value: u64,

    /// `user_id` presence announcement interval.
    ///
    /// Intervals at which `user_id` presence should be announced and should
    /// follow this optimal formula: `heartbeat_value / 2 - 1`.
    #[cfg(any(feature = "subscribe", feature = "presence"))]
    pub heartbeat_interval: Option<u64>,
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
    /// Returns [`PubNubClientRuntimeBuilder`] where depending from enabled
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
    /// let client = PubNubClientBuilder::with_transport(transport)
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
    /// Returns [`PubNubClientDeserializerBuilder`] where depending from enabled
    /// `features` following can be set:
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
    /// let client = PubNubClientBuilder::with_transport(transport)
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
    /// Returns [`PubNubClientRuntimeBuilder`] where depending from enabled
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
    /// let client = PubNubClientBuilder::with_blocking_transport(transport)
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
    /// Returns [`PubNubClientDeserializerBuilder`] where depending from enabled
    /// `features` following can be set:
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
    /// let client = PubNubClientBuilder::with_blocking_transport(transport)
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
/// Depending from enabled `features` methods may return:
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
    /// Returns [`PubNubClientDeserializerBuilder`] where depending from enabled
    /// `features` following can be set:
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
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let client = PubNubClientBuilder::with_reqwest_transport()
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
    /// # }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let client = PubNubClientBuilder::with_reqwest_transport()
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
    /// Set UUID for the client
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
                retry_policy: Default::default(),

                #[cfg(any(feature = "subscribe", feature = "presence"))]
                heartbeat_value: 300,

                #[cfg(any(feature = "subscribe", feature = "presence"))]
                heartbeat_interval: None,
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
            retry_policy: Default::default(),
            #[cfg(any(feature = "subscribe", feature = "presence"))]
            heartbeat_value: 300,
            #[cfg(any(feature = "subscribe", feature = "presence"))]
            heartbeat_interval: None,
        };

        assert!(config.signature_key_set().is_err());
    }
}
