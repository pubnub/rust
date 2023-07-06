//! PubNub client module
//!
//! This module contains the [`PubNubClient`] struct.
//! It's used to send requests to [`PubNub API`].
//! It's intended to be used by the [`pubnub`] crate.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`pubnub`]: ../index.html

use crate::core::Cryptor;
#[cfg(feature = "subscribe")]
use crate::dx::subscribe::subscription_manager::SubscriptionManager;
use crate::{
    core::{PubNubError, RequestRetryPolicy, Transport},
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

/// PubNub client
///
/// Client for PubNub API with support for all [`selected`] PubNub features.
/// The client is transport-layer-agnostic, so you can use any transport layer
/// that implements the [`Transport`] trait.
///
/// You can create clients using the [`PubNubClient::with_transport`]
/// You must provide a valid [`Keyset`] with pub/sub keys and a string User ID to identify the client.
///
/// To see available methods, please refer to the [`PubNubClientInstance`] documentation.
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
pub type PubNubGenericClient<T> = PubNubClientInstance<PubNubMiddleware<T>>;

/// PubNub client
///
/// Client for PubNub API with support for all [`selected`] PubNub features.
/// The client uses [`reqwest`] as a transport layer.
///
/// You can create clients using the [`PubNubClient::with_reqwest_transport`] method.
/// You must provide a valid [`Keyset`] with pub/sub keys and a string User ID to identify the client.
///
/// To see available methods, please refer to the [`PubNubClientInstance`] documentation.
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
/// [Keyset](struct.Keyset.html)
/// [Transport](../core/trait.Transport.html)
///
/// [`selected`]: ../index.html#features
/// [`Transport`]: ../core/trait.Transport.html
/// [`Keyset`]: ../core/struct.Keyset.html
/// [`reqwest`]: https://crates.io/crates/reqwest
/// [`PubNubClient::with_reqwest_transport`]: struct.PubNubClientBuilder.html#method.with_reqwest_transport
#[cfg(feature = "reqwest")]
pub type PubNubClient = PubNubGenericClient<crate::transport::TransportReqwest>;

/// PubNub client raw instance.
///
/// This struct contains the actual client state.
/// It shouldn't be used directly. Use [`PubNubGenericClient`] or [`PubNubClient`] instead.
pub struct PubNubClientInstance<T> {
    pub(crate) inner: Arc<PubNubClientRef<T>>,
}

impl<T> Deref for PubNubClientInstance<T> {
    type Target = PubNubClientRef<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for PubNubClientInstance<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to PubNubClientInstance are not allowed")
    }
}

impl<T> Clone for PubNubClientInstance<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Client reference
///
/// This struct contains the actual client state.
/// It's wrapped in `Arc` by [`PubNubClient`] and uses interior mutability for its internal state.
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
pub struct PubNubClientRef<T> {
    /// Transport layer
    pub(crate) transport: T,

    /// Data cryptor / decryptor
    #[builder(
        setter(custom, strip_option),
        field(vis = "pub(crate)"),
        default = "None"
    )]
    pub(crate) cryptor: Option<Arc<dyn Cryptor + Send + Sync>>,

    /// Instance ID
    #[builder(
        setter(into),
        field(type = "String", build = "Arc::new(Some(self.instance_id))")
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

    /// Subscription manager
    #[cfg(feature = "subscribe")]
    #[builder(setter(skip), field(vis = "pub(crate)"))]
    pub(crate) subscription_manager: Arc<RwLock<Option<SubscriptionManager>>>,
}

impl<T> PubNubClientInstance<T> {
    /// Create a new builder for [`PubNubClient`]
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse, PubNubError};
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl pubnub::core::blocking::Transport for MyTransport {
    /// #     fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// #  }
    /// #
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
    /// let builder = PubNubClientBuilder::with_blocking_transport(transport)
    ///     .with_keyset(Keyset {
    ///        publish_key: Some("pub-c-abc123"),
    ///        subscribe_key: "sub-c-abc123",
    ///        secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "blocking")]
    pub fn with_blocking_transport(transport: T) -> PubNubClientBuilder<T>
    where
        T: crate::core::blocking::Transport + Send + Sync,
    {
        PubNubClientBuilder {
            transport: Some(transport),
        }
    }

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
        token.is_empty().then_some(token)
    }
}

impl<T> PubNubClientConfigBuilder<T> {
    /// Set client authentication key.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part the
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

    /// Requests retry policy.
    ///
    /// The retry policy regulates the frequency of request retry attempts and the number of failed
    /// attempts that should be retried.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part the
    /// [`PubNubClientConfigBuilder`].
    pub fn with_retry_policy(mut self, policy: RequestRetryPolicy) -> Self {
        if let Some(configuration) = self.config.as_mut() {
            configuration.retry_policy = Arc::new(policy);
        }

        self
    }

    #[cfg(feature = "aescbc")]
    /// Data encryption / decryption
    ///
    /// Cryptor used by client when publish messages / signals and receive them as real-time updates
    /// from subscription module.
    ///
    /// It returns [`PubNubClientConfigBuilder`] that you can use to set the
    /// configuration for the client. This is a part the
    /// [`PubNubClientConfigBuilder`].
    pub fn with_cryptor<C>(mut self, cryptor: C) -> Self
    where
        C: Cryptor + Send + Sync + 'static,
    {
        self.cryptor = Some(Some(Arc::new(cryptor)));

        self
    }

    /// Build a [`PubNubClient`] from the builder
    pub fn build(self) -> Result<PubNubClientInstance<PubNubMiddleware<T>>, PubNubError> {
        self.build_internal()
            .map_err(|err| PubNubError::ClientInitialization {
                details: err.to_string(),
            })
            .and_then(|pre_build| {
                let token = Arc::new(RwLock::new(String::new()));
                info!("Client Configuration: \n publish_key: {:?}\n subscribe_key: {}\n user_id: {}\n instance_id: {:?}", pre_build.config.publish_key, pre_build.config.subscribe_key, pre_build.config.user_id, pre_build.instance_id);
                Ok(PubNubClientRef {
                    transport: PubNubMiddleware {
                        signature_keys: pre_build.config.clone().signature_key_set()?,
                        auth_key: pre_build.config.auth_key.clone(),
                        instance_id: pre_build.instance_id.clone(),
                        user_id: pre_build.config.user_id.clone(),
                        transport: pre_build.transport,
                        auth_token: token.clone(),
                    },
                    instance_id: pre_build.instance_id,
                    next_seqn: pre_build.next_seqn,
                    auth_token: token,
                    config: pre_build.config,
                    cryptor: pre_build.cryptor.clone(),

                    #[cfg(feature = "subscribe")]
                    subscription_manager: Arc::new(RwLock::new(None)),
                })
            })
            .map(|client| {
                PubNubClientInstance {
                    inner: Arc::new(client),
                }
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

    /// User ID
    pub(crate) user_id: Arc<String>,

    /// Publish key
    pub(crate) publish_key: Option<String>,

    /// Secret key
    pub(crate) secret_key: Option<String>,

    /// Authorization key
    pub(crate) auth_key: Option<Arc<String>>,

    /// Request retry policy
    pub(crate) retry_policy: Arc<RequestRetryPolicy>,
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
/// with the default transport layer. The `default` method is implemented only
/// when the `reqwest` feature is enabled.
///
/// The builder provides methods to set the transport layer and returns the next step
/// of the builder with the remaining parameters.
///
/// See [`PubNubClient`] for more information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientBuilder<T> {
    pub(crate) transport: Option<T>,
}

impl<T> Default for PubNubClientBuilder<T> {
    fn default() -> Self {
        Self { transport: None }
    }
}

impl<T> PubNubClientBuilder<T> {
    /// Set the transport layer for the client
    ///
    /// # Examples
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let client = PubNubClientBuilder::with_transport(transport)
    ///                     .with_keyset(Keyset {
    ///                         publish_key: Some("pub-c-abc123"),
    ///                         subscribe_key: "sub-c-abc123",
    ///                         secret_key: None,
    ///                     })
    ///                     .with_user_id("my-user-id")
    ///                     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_transport(transport: T) -> PubNubClientBuilder<T>
    where
        T: Transport,
    {
        PubNubClientBuilder {
            transport: Some(transport),
        }
    }

    /// Set the blocking transport layer for the client
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// # use pubnub::core::{blocking::Transport, TransportRequest, TransportResponse, PubNubError};
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
    /// # fn main() -> Result<(), PubNubError> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let client = PubNubClientBuilder::with_blocking_transport(transport)
    ///                     .with_keyset(Keyset {
    ///                         publish_key: Some("pub-c-abc123"),
    ///                         subscribe_key: "sub-c-abc123",
    ///                         secret_key: None,
    ///                     })
    ///                     .with_user_id("my-user-id")
    ///                     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "blocking")]
    pub fn with_blocking_transport(transport: T) -> PubNubClientBuilder<T>
    where
        T: crate::core::blocking::Transport + Send + Sync,
    {
        PubNubClientBuilder {
            transport: Some(transport),
        }
    }

    /// Set the keyset for the client
    ///
    /// It returns [`PubNubClientUserIdBuilder`] builder that you can use
    /// to set the User ID for the client.
    ///
    /// See [`Keyset`] for more information.
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// // note that with_reqwest_transport is only available when
    /// // the `reqwest` feature is enabled (default)
    /// let builder = PubNubClientBuilder::with_reqwest_transport()
    ///  .with_keyset(Keyset {
    ///    subscribe_key: "sub-c-abc123",
    ///    publish_key: Some("pub-c-abc123"),
    ///    secret_key: None,
    ///  });
    /// ```
    ///
    /// [`PubNubClientUserIdBuilder`]: struct.PubNubClientBuilderKeyset.html
    /// [`Keyset`]: struct.Keyset.html
    pub fn with_keyset<S>(self, keyset: Keyset<S>) -> PubNubClientUserIdBuilder<T, S>
    where
        S: Into<String>,
    {
        PubNubClientUserIdBuilder {
            transport: self.transport,
            keyset,
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientUserIdBuilder<T, S>
where
    S: Into<String>,
{
    transport: Option<T>,
    keyset: Keyset<S>,
}

impl<T, S> PubNubClientUserIdBuilder<T, S>
where
    S: Into<String>,
{
    /// Set UUID for the client
    /// It returns [`PubNubClientConfigBuilder`] that you can use
    /// to set the configuration for the client. This is a part
    /// the PubNubClientConfigBuilder.
    ///
    /// [`PubNubClientConfigBuilder`]: struct.PubNubClientConfigBuilder.html
    pub fn with_user_id<U>(self, user_id: U) -> PubNubClientConfigBuilder<T>
    where
        U: Into<String>,
    {
        let publish_key = self.keyset.publish_key.map(|k| k.into());
        let secret_key = self.keyset.secret_key.map(|k| k.into());

        PubNubClientConfigBuilder {
            transport: self.transport,
            config: Some(PubNubConfig {
                publish_key,
                subscribe_key: self.keyset.subscribe_key.into(),
                secret_key,
                user_id: Arc::new(user_id.into()),
                auth_key: None,
                retry_policy: Arc::new(RequestRetryPolicy::None),
            }),
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
///
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
    use crate::lib::alloc::boxed::Box;
    use std::any::type_name;

    #[test]
    fn include_pubnub_middleware() {
        #[derive(Default)]
        struct MockTransport;

        #[async_trait::async_trait]
        impl Transport for MockTransport {
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

        let client = PubNubClientBuilder::with_transport(MockTransport::default())
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
            retry_policy: Arc::new(Default::default()),
        };

        assert!(config.signature_key_set().is_err());
    }
}
