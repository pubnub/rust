//! TODO: Add documentation
use crate::PubNubClient;
use derive_builder::Builder;
use pubnub_core::{PubNubError, Transport, TransportMethod, TransportRequest};
use std::collections::HashMap;

/// TODO: Add documentation
pub type MessageType = String;

/// TODO: Add documentation
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

        let request = if instance.use_post == true {
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
    use crate::PubNubClient;
    use pubnub_core::{transport_response::TransportResponse, Transport};

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

    fn test<T>(instance: PubNubClient<T>)
    where
        T: Transport,
    {
        instance
            .publish_message("First message".into())
            .channel("Iguess".into())
            .replicate(true)
            .execute();

        instance
            .publish_message("I could send this message".into())
            .channel("chan".into())
            .store(true)
            .ttl(140)
            .execute();
    }
}
