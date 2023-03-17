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
/// use pubnub_client::{PubNubClient, Keyset};
///
/// // note that `default` method is implemented only when the `reqwest` feature is enabled
/// let client = PubNubClientBuilder::default()
///    .with_keyset(Keyset {
///         publish_key: "pub-c-abc123",
///         subscribe_key: "sub-c-abc123",
///         secret_key: None,
///    })
///    .with_user_id("my-user-id")
///    .build();
/// ```
///
/// or using own transport implementation
///
/// ```
/// use pubnub_client::{PubNubClient, Keyset};
///
/// // your implemented transport that implements `Transport` trait
/// let transport = MyTransport::new();
///
/// let client = PubNubClient::builder()
///    .with_transport(MyTransport)
///    .with_keyset(Keyset {
///         publish_key: "pub-c-abc123",
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
    pub(crate) next_seqn: u16,

    /// Subscribe key
    pub(crate) subscribe_key: String,

    /// User ID
    pub(crate) user_id: String,

    /// Publish key
    #[builder(setter(strip_option), default = "None")]
    pub(crate) publish_key: Option<String>,

    /// Secret key
    #[builder(setter(strip_option), default = "None")]
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
    transport: Option<T>,
}

impl<T> PubNubClientBuilder<T>
where
    T: Transport,
{
    /// Create a new builder with default transport
    ///
    /// # Examples
    /// ```
    /// use pubnub_client::PubNubClientBuilder;
    ///
    /// let builder = PubNubClientBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self { transport: None }
    }

    /// Set transport for the client
    ///
    /// # Examples
    /// ```
    /// use pubnub_client::{PubNubClientBuilder, Keyset};
    /// use reqwest::Client;
    ///
    /// let builder = PubNubClientBuilder::new()
    ///   .with_transport(Client::new());
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
    /// use pubnub_client::{PubNubClientBuilder, Keyset};
    ///
    /// let builder = PubNubClientBuilder::default()
    ///  .with_keyset(Keyset {
    ///    subscribe_key: "sub-c-abc123",
    ///    publish_key: "pub-c-abc123",
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
/// use pubnub_client::{PubNubClientBuilder, Keyset};
///
/// let builder = PubNubClientBuilder::default()
/// .with_keyset(Keyset {
///     subscribe_key: "sub-c-abc123",
///     publish_key: "pub-c-abc123",
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
            publish_key: Some(publish_key),
            subscribe_key: Some(self.keyset.subscribe_key.into()),
            secret_key: Some(secret_key),
            user_id: Some(user_id.into()),
            ..Default::default()
        }
    }
}

/// Keyset for PubNub client
///
/// # Examples
/// ```
/// use pubnub_client::Keyset;
///
/// Keyset {
///    subscribe_key: "sub-c-abc123",
///    publish_key: "pub-c-abc123",
///    secret_key: "sec-c-abc123",
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
