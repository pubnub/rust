//! # Publish builders module.
//!
//! This module contains all builders for the publish operation.

use derive_builder::Builder;

use crate::{
    core::Serialize,
    dx::pubnub_client::PubNubClientInstance,
    lib::{alloc::string::String, collections::HashMap},
};

/// The [`PublishMessageBuilder`] is used to publish a message to a channel.
///
/// This struct is used by the [`publish_message`] method of the
/// [`PubNubClient`].
/// The [`publish_message`] method is used to publish a message to a channel.
/// The [`PublishMessageBuilder`] is used to build the request that is sent to
/// the [`PubNub`] network.
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
/// [`publish_message`]: crate::dx::PubNubClient::publish_message`
/// [`PubNubClient`]: crate::dx::PubNubClient
/// [`PubNub`]:https://www.pubnub.com/
pub struct PublishMessageBuilder<T, M, D>
where
    M: Serialize,
{
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    pub(super) pub_nub_client: PubNubClientInstance<T, D>,
    pub(super) message: M,
    pub(super) seqn: u16,
}

impl<T, M, D> PublishMessageBuilder<T, M, D>
where
    M: Serialize,
{
    /// The [`channel`] method is used to set the channel to publish the message
    /// to.
    ///
    /// [`channel`]: crate::dx::publish::PublishMessageBuilder::channel
    pub fn channel<S>(self, channel: S) -> PublishMessageViaChannelBuilder<T, M, D>
    where
        S: Into<String>,
    {
        PublishMessageViaChannelBuilder::<T, M, D> {
            pub_nub_client: Some(self.pub_nub_client),
            seqn: Some(self.seqn),
            ..Default::default()
        }
        .message(self.message)
        .channel(channel.into())
    }
}

/// The [`PublishMessageViaChannelBuilder`] is is next step in the publish
/// process.
/// The [`PublishMessageViaChannelBuilder`] is used to build the request to be
/// sent to the [`PubNub`] network.
/// This struct is used to publish a message to a channel. It is used by the
/// [`publish_message`] method of the [`PubNubClient`].
///
/// # Examples
/// ```rust
/// # use pubnub::{Keyset, PubNubClientBuilder};
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
/// [`publish_message`]: crate::dx::PubNubClient::publish_message
/// [`PubNubClient`]: crate::dx::PubNubClient
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(vis = "pub(super)"))]
#[cfg_attr(not(feature = "std"), builder(no_std))]
pub struct PublishMessageViaChannel<T, M, D>
where
    M: Serialize,
{
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(setter(custom))]
    pub(super) pub_nub_client: PubNubClientInstance<T, D>,

    #[builder(setter(custom))]
    pub(super) seqn: u16,

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
    pub(super) custom_message_type: Option<String>,
}
