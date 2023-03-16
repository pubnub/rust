//! # Publish module
//!
//! This module provides feature that allows to publish messages to PubNub channels.
//! It is intended to be used by the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html
//! [PubNub]: https://www.pubnub.com/
//! [Publish API]: https://www.pubnub.com/docs/rest-api/endpoints/publish
//! [Publish API documentation]: https://www.pubnub.com/docs/rest-api/endpoints/publish

use crate::{
    core::{PubNubError, Transport, TransportMethod, TransportRequest},
    dx::PubNubClient,
};
use derive_builder::Builder;
use std::collections::HashMap;

/// TODO: Add documentation
pub type MessageType = String;

/// This struct is a step of the publish message builder.
/// It is used to specify the channel to publish the message to.
/// It is created by the [`publish_message`] method.
/// It is completed by the [`channel`] method.
///
/// [`publish_message`]: struct.PubNubClient.html#method.publish_message
/// [`channel`]: #method.channel
pub struct PublishMessageBuilder<'pub_nub, T>
where
    T: Transport,
{
    pub_nub_client: &'pub_nub PubNubClient<T>,
    message: MessageType,
}

impl<'pub_nub, T> PublishMessageBuilder<'pub_nub, T>
where
    T: Transport,
{
    /// TODO: Add documentation
    pub fn channel(self, channel: String) -> PublishMessageViaChannelBuilder<'pub_nub, T> {
        PublishMessageViaChannelBuilder {
            pub_nub_client: Some(self.pub_nub_client),
            ..Default::default()
        }
        .message(self.message)
        .channel(channel)
    }
}

/// TODO: Add documentation
// TODO: use dead codes
#[allow(dead_code)]
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(private))]
pub struct PublishMessageViaChannel<'pub_nub, T>
where
    T: Transport,
{
    #[builder(setter(custom))]
    pub_nub_client: &'pub_nub PubNubClient<T>,
    /// TODO: Add documentation
    message: MessageType,
    /// TODO: Add documentation
    channel: String,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default)]
    store: Option<bool>,
    /// TODO: Add documentation
    #[builder(default = "true")]
    replicate: bool,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default)]
    ttl: Option<u32>,
    /// TODO: Add documentation
    #[builder(default = "false")]
    use_post: bool,
    /// TODO: Add documentation
    #[builder(default = "HashMap::new()")]
    meta: HashMap<String, String>,
}

impl<'pub_nub, T> PublishMessageViaChannelBuilder<'pub_nub, T>
where
    T: Transport,
{
    /// TODO: Add documentation
    pub async fn execute(self) -> Result<PublishResult, PubNubError> {
        let instance = self
            .build()
            .map_err(|err| PubNubError::PublishError(err.to_string()))?;

        let pub_key = "";
        let sub_key = "";

        let request = if instance.use_post {
            TransportRequest {
                path: format!("publish/{sub_key}/{pub_key}/0/{}/0", instance.channel),
                method: TransportMethod::Post,
                //body: self.message.unwrap(), TODO
                ..Default::default()
            }
        } else {
            TransportRequest {
                path: format!(
                    "publish/{}/{}/0/{}/0/\"{}\"",
                    sub_key, pub_key, instance.channel, instance.message
                ),
                method: TransportMethod::Get,
                ..Default::default()
            }
        };

        instance
            .pub_nub_client
            .transport
            .send(request)
            .await
            .map(|_| PublishResult)
    }
}

/// TODO: Add documentation
#[derive(Debug)]
pub struct PublishResult;

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// TODO: Add documentation
    pub fn publish_message(&self, message: MessageType) -> PublishMessageBuilder<T> {
        PublishMessageBuilder {
            message,
            pub_nub_client: self,
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::TransportResponse, dx::PubNubClient};

    #[tokio::test]
    async fn publish_message() {
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

        let client = PubNubClient {
            transport: MockTransport::default(),
        };

        let result = client
            .publish_message("First message".into())
            .channel("Iguess".into())
            .replicate(true)
            .execute()
            .await;

        assert!(dbg!(result).is_ok());
    }
}
