//! PubNub client module
//!
//! This module contains the [`PubNubClient`] struct.
//! It's used to send requests to [`PubNub API`].
//! It's intended to be used by the [`pubnub`] crate.
//!
//! [`PubNubClient`]: ./struct.PubNubClient.html]
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`pubnub`]: ../index.html

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::core::PubNubError::ClientInitializationError;
use crate::transport::middleware::SignatureKeySet;
use crate::{core::PubNubError, core::Transport, transport::middleware::PubNubMiddleware};
use derive_builder::Builder;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const SDK_ID: &str = "PubNub-Rust";

/// PubNub client
///
/// Client for PubNub API with support for all [`selected`] PubNub features.
/// The client is transport-layer-agnostic, so you can use any transport layer
/// that implements the [`Transport`] trait.
///
/// You can create clients using the [`PubNubClient::builder`] method.
/// You must provide a valid [`Keyset`] with pub/sub keys and a string User ID to identify the client.
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
/// use pubnub::{PubNubClient, Keyset};
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
/// # fn main() -> Result<(), pubnub::core::PubNubError> {
/// // note that MyTransport must implement the `Transport` trait
/// let transport = MyTransport::new();
///
/// let client = PubNubClient::with_transport(MyTransport)
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
/// [`PubNubClient::builder`]: ./struct.PubNubClient.html#method.builder
#[derive(Debug)]
pub struct PubNubClient<'a, T>
where
    T: Transport,
{
    // TODO: how about Rc and blocking calls?
    //       or should we be always async library?
    pub(crate) inner: Arc<PubNubClientRef<'a, T>>,
}

/// Client reference
///
/// This struct contains the actual client state.
/// It's wrapped in `Arc` by [`PubNubClient`] and uses interior mutability for its internal state.
///
/// Not intended to be used directly. Use [`PubNubClient`] instead.
///
/// [`PubNubClient`]: ./struct.PubNubClient.html
#[derive(Debug, Builder)]
#[builder(
    pattern = "owned",
    name = "PubNubClientConfigBuilder",
    build_fn(private, name = "build_internal"),
    setter(prefix = "with")
)]
pub struct PubNubClientRef<'a, T>
where
    T: Transport,
{
    /// Transport layer
    pub(crate) transport: T,

    /// Instance ID
    #[builder(setter(strip_option), default = "None")]
    pub(crate) instance_id: Option<String>,

    /// Sequence number for the publish requests
    #[builder(default = "Mutex::new(1)")]
    pub(crate) next_seqn: Mutex<u16>,

    /// Configuration
    pub(crate) config: PubNubConfig<'a>,
}

impl<T> PubNubClient<'_, T>
where
    T: Transport,
{
    /// Create a new builder for [`PubNubClient`]
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClient, Keyset};
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
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let builder = PubNubClient::with_transport(transport)
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
    ///
    /// [`PubNubClient`]: struct.PubNubClient.html
    pub fn with_transport(transport: T) -> PubNubClientBuilder<T> {
        PubNubClientBuilder {
            transport: Some(transport),
        }
    }
}

impl<'a, T> Deref for PubNubClient<'a, T>
where
    T: Transport,
{
    type Target = PubNubClientRef<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for PubNubClient<'_, T>
where
    T: Transport,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> PubNubClientConfigBuilder<'static, T>
where
    T: Transport,
{
    /// Build a [`PubNubClient`] from the builder
    ///
    /// [`PubNubClient`]: struct.PubNubClient.html
    pub fn build(self) -> Result<PubNubClient<'static, PubNubMiddleware<'static, T>>, PubNubError<'static>> {
        self.build_internal()
            .map_err(|err| ClientInitializationError(&err.to_string()))
            .and_then(|pre_build| {
                Ok(PubNubClientRef {
                    transport: PubNubMiddleware {
                        transport: pre_build.transport,
                        // TODO: String -> Cow<'static, str>
                        instance_id: pre_build.instance_id.clone(),
                        user_id: pre_build.config.user_id.clone(),
                        signature_keys: pre_build.config.clone().signature_key_set()?,
                    },
                    instance_id: pre_build.instance_id,
                    next_seqn: pre_build.next_seqn,
                    config: pre_build.config,
                })
            })
            .map(|client| PubNubClient {
                inner: Arc::new(client),
            })
    }
}

/// PubNub configuration
///
/// Configuration for [`PubNubClient`].
/// This struct separates the configuration from the actual client.
///
/// [`PubNubClient`]: struct.PubNubClient.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubConfig<'a> {
    /// Subscribe key
    pub(crate) subscribe_key: String,

    /// User ID
    pub(crate) user_id: String,

    /// Publish key
    pub(crate) publish_key: Option<&'a str>,

    /// Secret key
    pub(crate) secret_key: Option<&'a str>,
}

impl<'a> PubNubConfig <'a>{
    fn signature_key_set(self) -> Result<Option<SignatureKeySet<'a>>, PubNubError<'a>> {
        if let Some(secret_key) = self.secret_key {
            let publish_key = self.publish_key.ok_or(ClientInitializationError(
                "You must also provide the publish key if you use the secret key.",
            ))?;
            Ok(Some(SignatureKeySet {
                secret_key,
                publish_key,
                subscribe_key: &self.subscribe_key,
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
///
/// [`PubNubClient`]: struct.PubNubClient.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientBuilder<T>
where
    T: Transport<>,
{
    pub(crate) transport: Option<T>,
}

impl<T> Default for PubNubClientBuilder<T>
where
    T: Transport,
{
    fn default() -> Self {
        Self { transport: None }
    }
}

impl<T> PubNubClientBuilder<T>
where
    T: Transport,
{
    /// Create a new builder without transport
    ///
    /// # Examples
    /// ```
    /// use pubnub::PubNubClientBuilder;
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
    /// // note that MyTransport must implement the `Transport` trait
    /// let builder = PubNubClientBuilder::<MyTransport>::new();
    /// ```
    ///
    /// [`Transport`]: ../core/trait.Transport.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the transport layer for the client
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClient, Keyset};
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
    /// // note that MyTransport must implement the `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let client = PubNubClient::with_transport(transport);
    /// ```
    pub fn with_transport<U>(self, transport: U) -> PubNubClientBuilder<U>
    where
        U: Transport,
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
/// [Keyset](struct.Keyset.html)
/// [PubNubClientBuilder](struct.PubNubClientBuilder.html)
/// [PubNubClient](struct.PubNubClient.html)
///
/// [`PubNubClient`]: struct.PubNubClient.html
/// [`PubNubClientBuilder::with_keyset`]: struct.PubNubClientBuilder.html#method.with_keyset
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientUserIdBuilder<T, S>
where
    T: Transport,
    S: Into<String>,
{
    transport: Option<T>,
    keyset: Keyset<S>,
}

impl<T, S> PubNubClientUserIdBuilder<T, S>
where
    T: Transport,
    S: Into<String>,
{
    /// Set UUID for the client
    /// It returns [`PubNubClientConfigBuilder`] that you can use
    /// to set the configuration for the client. This is a part
    /// the PubNubClientConfigBuilder.
    ///
    /// [`PubNubClientConfigBuilder`]: struct.PubNubClientConfigBuilder.html
    /// [`PubNubClient`]: struct.PubNubClient.html
    pub fn with_user_id<U>(self, user_id: U) -> PubNubClientConfigBuilder<'static, T>
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
                user_id: user_id.into()
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

        let client = PubNubClient::with_transport(MockTransport::default())
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
            user_id: "".into(),
        };

        assert!(config.signature_key_set().is_err());
    }
}
