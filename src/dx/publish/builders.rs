//! Publish builders module.
//!
//! This module contains all builders for the publish operation.

use super::PublishResponseBody;
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;
use crate::{
    core::{Deserializer, Serialize},
    dx::pubnub_client::PubNubClientInstance,
    lib::{alloc::string::String, collections::HashMap},
};
use derive_builder::Builder;

/// The [`PublishMessageBuilder`] is used to publish a message to a channel.
///
/// This struct is used by the [`publish_message`] method of the [`PubNubClient`].
/// The [`publish_message`] method is used to publish a message to a channel.
/// The [`PublishMessageBuilder`] is used to build the request that is sent to the [`PubNub`] network.
///
/// # Examples
/// ```rust
/// # use pubnub::{PubNubClientBuilder, Keyset};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut pubnub = // PubNubClient
/// # PubNubClientBuilder::with_reqwest_transport()
/// #     .with_keyset(Keyset{
/// #         subscribe_key: "demo",
/// #         publish_key: Some("demo"),
/// #         secret_key: None,
/// #     })
/// #     .with_user_id("user_id")
/// #     .build()?;
///
/// pubnub.publish_message("hello world!")
///     .channel("my_channel")
///     .execute()
///     .await?;
///
/// # Ok(())
/// # }
/// ```
///
/// [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder]
/// [`publish_message`]: crate::dx::PubNubClient::publish_message`
/// [`PubNubClient`]: crate::dx::PubNubClient
/// [`PubNub`]:https://www.pubnub.com/
pub struct PublishMessageBuilder<T, M>
where
    M: Serialize,
{
    pub(super) pub_nub_client: PubNubClientInstance<T>,
    pub(super) message: M,
    pub(super) seqn: u16,
}

impl<T, M> PublishMessageBuilder<T, M>
where
    M: Serialize,
{
    /// The [`channel`] method is used to set the channel to publish the message to.
    ///
    /// [`channel`]: crate::dx::publish::PublishMessageBuilder::channel
    #[cfg(feature = "serde")]
    pub fn channel<S>(self, channel: S) -> PublishMessageViaChannelBuilder<T, M, SerdeDeserializer>
    where
        S: Into<String>,
    {
        PublishMessageViaChannelBuilder::<T, M, SerdeDeserializer> {
            pub_nub_client: Some(self.pub_nub_client),
            seqn: Some(self.seqn),
            ..Default::default()
        }
        .message(self.message)
        .channel(channel.into())
        .deserialize_with(SerdeDeserializer)
    }

    /// The [`channel`] method is used to set the channel to publish the message to.
    ///
    /// [`channel`]: crate::dx::publish::PublishMessageBuilder::channel

    #[cfg(not(feature = "serde"))]
    pub fn channel<S>(self, channel: S) -> PublishMessageDeserializerBuilder<T, M>
    where
        S: Into<String>,
    {
        PublishMessageDeserializerBuilder {
            pub_nub_client: self.pub_nub_client,
            message: self.message,
            seqn: self.seqn,
            channel: channel.into(),
        }
    }
}

/// The [`PublishMessageDeserializer`] adds the deserializer to the [`PublishMessageBuilder`].
///
/// This struct is used to publish a message to a channel. It is used by the [`publish_message`] method of the [`PubNubClient`].
///
/// The [`publish_message`] method is used to publish a message to a channel.
///
/// See more information in the [`PublishMessageBuilder`] struct and the [`Deserializer`] trait.
///
/// # Examples
/// ```rust
/// # use pubnub::{PubNubClientBuilder, Keyset};
/// use pubnub::{
///     dx::publish::PublishResponse,
///     core::{Deserializer, PubNubError}
/// };
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// struct MyDeserializer;
///
/// impl<'de> Deserializer<'de, PublishResponseBody> for MyDeserializer {
///    fn deserialize(&self, response: &'de [u8]) -> Result<PublishResponse, PubNubError> {
///    // ...
///    # Ok(PublishResponse)
/// }
///
///
/// let mut pubnub = // PubNubClient
/// # PubNubClientBuilder::with_reqwest_transport()
/// #     .with_keyset(Keyset{
/// #         subscribe_key: "demo",
/// #         publish_key: Some("demo"),
/// #         secret_key: None,
/// #     })
/// #     .with_user_id("user_id")
/// #     .build()?;
///
/// pubnub.publish_message("hello world!")
///    .channel("my_channel")
///    .deserialize_with(MyDeserializer)
///    .execute()
///    .await?;
/// # Ok(())
/// # }
/// ```
///
/// [`PublishMessageDeserializer`]: crate::dx::publish::PublishMessageDeserializer
/// [`publish_message`]: crate::dx::PubNubClient::publish_message`
/// [`PubNubClient`]: crate::dx::PubNubClient
/// [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder
/// [`Deserializer`]: crate::core::Deserializer
#[cfg(not(feature = "serde"))]
pub struct PublishMessageDeserializerBuilder<T, M>
where
    M: Serialize,
{
    pub_nub_client: PubNubClientInstance<T>,
    message: M,
    seqn: u16,
    channel: String,
}

#[cfg(not(feature = "serde"))]
impl<T, M> PublishMessageDeserializerBuilder<T, M>
where
    M: Serialize,
{
    /// The [`deserialize_with`] method is used to set the deserializer to deserialize the response with.
    /// It's important to note that the deserializer must implement the [`Deserializer`] trait for
    /// the [`PublishResponse`] type.
    ///
    /// [`deserialize_with`]: crate::dx::publish::PublishMessageDeserializerBuilder::deserialize_with
    /// [`Deserializer`]: crate::core::Deserializer
    /// [`PublishResponseBody`]: crate::core::publish::PublishResponseBody
    pub fn deserialize_with<D>(self, deserializer: D) -> PublishMessageViaChannelBuilder<T, M, D>
    where
        for<'de> D: Deserializer<'de, PublishResponseBody>,
    {
        PublishMessageViaChannelBuilder {
            pub_nub_client: Some(self.pub_nub_client),
            seqn: Some(self.seqn),
            deserializer: Some(deserializer),
            ..Default::default()
        }
        .message(self.message)
        .channel(self.channel)
    }
}

/// The [`PublishMessageViaChannelBuilder`] is is next step in the publish process.
/// The [`PublishMessageViaChannelBuilder`] is used to build the request to be sent to the [`PubNub`] network.
/// This struct is used to publish a message to a channel. It is used by the [`publish_message`] method of the [`PubNubClient`].
///
/// # Examples
/// ```rust
/// # use pubnub::{PubNubClientBuilder, Keyset};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut pubnub = // PubNubClient
/// # PubNubClientBuilder::with_reqwest_transport()
/// #     .with_keyset(Keyset{
/// #         subscribe_key: "demo",
/// #         publish_key: Some("demo"),
/// #         secret_key: None,
/// #     })
/// #     .with_user_id("user_id")
/// #     .build()?;
///
/// pubnub.publish_message("hello world!")
///     .channel("my_channel")
///     .execute()
///     .await?;
///
/// # Ok(())
/// # }
/// ```
///
/// [`PublishMessageViaChannelBuilder`]: struct.PublishMessageViaChannelBuilder
/// [`publish_message`]: crate::dx::PubNubClient::publish_message
/// [`PubNub`]:https://www.pubnub.com/
/// [`PubNubClient`]: crate::dx::PubNubClient
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(vis = "pub(super)"))]
#[cfg_attr(not(feature = "std"), builder(no_std))]
pub struct PublishMessageViaChannel<T, M, D>
where
    M: Serialize,
    D: for<'de> Deserializer<'de, PublishResponseBody>,
{
    #[builder(setter(custom))]
    pub(super) pub_nub_client: PubNubClientInstance<T>,

    #[builder(setter(custom))]
    pub(super) seqn: u16,

    /// Deserializer to deserialize the response with.
    /// Note that the deserializer must implement the [`Deserializer`] trait for
    /// the [`PublishResponseBody`] type.
    /// [`Deserializer`]: crate::core::Deserializer
    /// [`PublishResponseBody`]: crate::core::publish::PublishResponseBody
    #[builder(setter(name = "deserialize_with"))]
    pub(super) deserializer: D,

    /// Message to publish
    pub(super) message: M,

    /// Channel to publish to
    #[builder(setter(into))]
    pub(super) channel: String,

    /// Switch that decides if the message should be stored in history
    #[builder(setter(strip_option), default = "None")]
    pub(super) store: Option<bool>,

    /// Switch that decides if the transaction should be replicated
    /// following the PubNub replication rules.
    ///
    /// See more at [`PubNub replication rules`]
    ///
    /// [`PubNub replication rules`]:https://www.pubnub.com/pricing/transaction-classification/
    #[builder(default = "true")]
    pub(super) replicate: bool,

    /// Set a per-message TTL time to live in Message Persistence.
    #[builder(setter(strip_option), default = "None")]
    pub(super) ttl: Option<u32>,

    /// Switch that decide if the message should be published using POST method.
    #[builder(setter(strip_option), default = "false")]
    pub(super) use_post: bool,

    /// Object to send additional information about the message.
    #[builder(setter(strip_option), default = "None")]
    pub(super) meta: Option<HashMap<String, String>>,

    /// Space ID to publish to.
    #[builder(setter(strip_option, into), default = "None")]
    pub(super) space_id: Option<String>,

    /// Message type to publish.
    #[builder(setter(strip_option, into), default = "None")]
    pub(super) r#type: Option<String>,
}
