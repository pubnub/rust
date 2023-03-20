//! TODO: Add documentation

use crate::core::Transport;
use derive_builder::Builder;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const SDK_ID: &str = "PubNub-Rust";

/// PubNub client
///
/// Client for PubNub API with support for all [selected] PubNub features.
/// It is generic over transport layer, which allows to use any transport
/// that implements [`Transport`] trait.
///
/// Client can be created using [`PubNubClient::builder`] method.
/// It is required to use [`Keyset`] to provide keys to the client
/// and UUID to identify the client.
///
/// [selected]: ../index.html#features
///
/// # Examples
/// ```
/// use pubnub::{PubNubClientBuilder, Keyset};
///
/// // note that `with_reqwest_transport` requires `reqwest` feature
/// // to be enabled (default)
/// let client = PubNubClientBuilder::with_reqwest_transport()
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build();
/// ```
///
/// or using own [`Transport`] implementation
///
/// [`Transport`]: ../core/trait.Transport.html
///
/// ```
/// use pubnub::{PubNubClient, Keyset};
///
/// # use pubnub::core::{Transport, TransportRequest, TransportResponse};
/// # struct MyTransport;
/// # #[async_trait::async_trait]
/// # impl Transport for MyTransport {
/// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse,
/// pubnub::PubNubError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # impl MyTransport {
/// #     fn new() -> Self {
/// #         Self
/// #     }
/// # }
///
/// // note that MyTransport must implement `Transport` trait
/// let transport = MyTransport::new();
///
/// let client = PubNubClient::with_transport(MyTransport)
///    .with_keyset(Keyset {
///         publish_key: Some("pub-c-abc123"),
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build();
/// ```
///
/// # See also
/// [Keyset](struct.Keyset.html)
/// [Transport](../core/trait.Transport.html)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Builder)]
#[builder(
    pattern = "owned",
    name = "PubNubClientConfigBuilder",
    setter(prefix = "with")
)]
pub struct PubNubClient<T>
where
    T: Transport,
{
    /// Transport layer
    pub(crate) transport: T,

    /// Sequence number for the publish requests
    #[builder(default = "1")]
    pub(crate) next_seqn: u16,

    /// Configuration
    pub(crate) config: PubNubConfig,
}

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// Create a new builder for [`PubNubClient`]
    ///
    /// [`PubNubClient`]: struct.PubNubClient.html
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClient, Keyset};
    ///
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse};
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl Transport for MyTransport {
    /// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, pubnub::PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// // note that MyTransport must implement `Transport` trait
    /// let transport = MyTransport::new();
    ///
    /// let builder = PubNubClient::with_transport(transport)
    ///     .with_keyset(Keyset {
    ///        publish_key: Some("pub-c-abc123"),
    ///        subscribe_key: "sub-c-abc123",
    ///        secret_key: None,
    ///     })
    ///     .with_user_id("my-user-id")
    ///     .build();
    /// ```
    pub fn with_transport(transport: T) -> PubNubClientBuilder<T> {
        PubNubClientBuilder {
            transport: Some(transport),
        }
    }
}

/// PubNub configuration
///
/// Configuration for [`PubNubClient`].
/// This struct exists to separate configuration from the client.
///
/// [`PubNubClient`]: struct.PubNubClient.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubConfig {
    /// Subscribe key
    pub(crate) subscribe_key: String,

    /// User ID
    pub(crate) user_id: String,

    /// Publish key
    pub(crate) publish_key: Option<String>,

    /// Secret key
    pub(crate) secret_key: Option<String>,
}

/// PubNub builder for [`PubNubClient`]
///
/// Builder for [`PubNubClient`] that is a first step to create a client.
/// It is generic over transport layer, which allows to use any transport
///
/// It is possible to use [`Default`] implementation to create a builder
/// with default transport. (Note that `default` method is implemented only
/// when the `reqwest` feature is enabled)
///
/// It provides methods to set transport and return next step builder
/// with rest of the parameters.
///
/// This design forces developers to configure PubNub Client
/// with all required parameters. It makes the configuration
/// more explicit and less error prone.
///
/// See [`PubNubClient`] for more information.
///
/// [`PubNubClient`]: struct.PubNubClient.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientBuilder<T>
where
    T: Transport,
{
    pub(crate) transport: Option<T>,
}

impl<T> PubNubClientBuilder<T>
where
    T: Transport,
{
    /// Create a new builder with without transport
    ///
    /// # Examples
    /// ```
    /// use pubnub::PubNubClientBuilder;
    ///
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse};
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl Transport for MyTransport {
    /// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse,
    /// pubnub::PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// // note that MyTransport must implement `Transport` trait
    /// let builder = PubNubClientBuilder::<MyTransport>::new();
    /// ```
    ///
    /// [`Transport`]: ../core/trait.Transport.html
    pub fn new() -> Self {
        Self { transport: None }
    }

    /// Set transport for the client
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClient, Keyset};
    ///
    /// # use pubnub::core::{Transport, TransportRequest, TransportResponse};
    /// # struct MyTransport;
    /// # #[async_trait::async_trait]
    /// # impl Transport for MyTransport {
    /// #     async fn send(&self, _request: TransportRequest) -> Result<TransportResponse,
    /// pubnub::PubNubError> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl MyTransport {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    ///
    /// // note that MyTransport must implement `Transport` trait
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

    /// Set keyset for the client
    ///
    /// It returns [`PubNubClientUuidBuilder`] builder that can be used
    /// to set UUID for the client.
    /// See [`Keyset`] for more information.
    ///
    /// [`PubNubClientUuidBuilder`]: struct.PubNubClientBuilderKeyset.html
    /// [`Keyset`]: struct.Keyset.html
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
    pub fn with_keyset<S>(self, keyset: Keyset<S>) -> PubNubClientUuidBuilder<T, S>
    where
        S: Into<String>,
    {
        PubNubClientUuidBuilder {
            transport: self.transport,
            keyset,
        }
    }
}

/// PubNub builder for [`PubNubClient`] for setting UUID
/// It is returned by [`PubNubClientBuilder::with_keyset`]
/// and provides method to set UUID for the client.
/// See [`PubNubClient`] for more information.
///
/// [`PubNubClient`]: struct.PubNubClient.html
/// [`PubNubClientBuilder::with_keyset`]: struct.PubNubClientBuilder.html#method.with_keyset
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubNubClientUuidBuilder<T, S>
where
    T: Transport,
    S: Into<String>,
{
    transport: Option<T>,
    keyset: Keyset<S>,
}

impl<T, S> PubNubClientUuidBuilder<T, S>
where
    T: Transport,
    S: Into<String>,
{
    /// Set UUID for the client
    /// It returns [`PubNubClientConfigBuilder`] that can be used
    /// to set configuration for the client. It assumes that
    /// every required parameters are set.
    ///
    /// [`PubNubClientConfigBuilder`]: struct.PubNubClientConfigBuilder.html
    /// [`PubNubClient`]: struct.PubNubClient.html
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
                user_id: user_id.into(),
            }),
            ..Default::default()
        }
    }
}

/// Keyset for PubNub client
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
/// # See also
/// [Keysets](https://www.pubnub.com/docs/platform/keys)
/// [Keyset management](https://www.pubnub.com/docs/platform/keyset-management)
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
