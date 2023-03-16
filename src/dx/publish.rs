//! TODO: Add documentation
use crate::{
    core::{PubNubError, Transport, TransportMethod, TransportRequest},
    dx::PubNubClient,
};
use derive_builder::Builder;
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
    #[builder(setter(strip_option), default = "None")]
    store: Option<bool>,
    /// TODO: Add documentation
    #[builder(default = "true")]
    replicate: bool,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default = "None")]
    ttl: Option<u32>,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default = "false")]
    use_post: bool,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default = "None")]
    meta: Option<HashMap<String, String>>,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default = "None")]
    space_id: Option<String>,
    /// TODO: Add documentation
    #[builder(setter(strip_option), default = "None")]
    message_type: Option<String>,
}

fn bool_to_numeric(value: bool) -> String {
    if value { "1" } else { "0" }.to_string()
}

fn prepare_publish_query_params<T>(
    publish_struct: &PublishMessageViaChannel<T>,
) -> HashMap<String, String>
where
    T: Transport,
{
    let mut query_params: HashMap<String, String> = HashMap::new();

    if let Some(store) = publish_struct.store {
        query_params.insert("store".to_string(), bool_to_numeric(store));
    }

    if let Some(ttl) = publish_struct.ttl {
        query_params.insert("ttl".to_string(), ttl.to_string());
    }

    if !publish_struct.replicate {
        query_params.insert("norep".to_string(), true.to_string());
    }

    if let Some(space_id) = publish_struct.space_id {
        query_params.insert("space-id".to_string(), space_id);
    }

    if let Some(message_type) = &publish_struct.message_type {
        query_params.insert("type".to_string(), message_type.clone());
    }

    query_params
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

        let query_params = prepare_publish_query_params(&instance);

        let request = if instance.use_post {
            TransportRequest {
                path: format!("publish/{sub_key}/{pub_key}/0/{}/0", instance.channel),
                method: TransportMethod::Post,
                query_parameters: query_params,
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
                query_parameters: query_params,
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

    #[tokio::test]
    async fn verify_all_query_parameters() {
        #[derive(Default)]
        struct MockTransport;

        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(
                &self,
                request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                assert_eq!(
                    HashMap::<String, String>::from([
                        ("norep".into(), "true".into()),
                        ("store".into(), "1".into()),
                        ("space-id".into(), "space_id".into()),
                        ("type".into(), "message_type".into()),
                        ("ttl".into(), "50".into())
                    ]),
                    request.query_parameters
                );
                Ok(TransportResponse::default())
            }
        }

        let client = PubNubClient {
            transport: MockTransport::default(),
        };

        let result = client
            .publish_message("message".into())
            .channel("chan".into())
            .replicate(false)
            .ttl(50)
            .store(true)
            .space_id("space_id".into())
            .message_type("message_type".into())
            .execute()
            .await;

        assert!(dbg!(result).is_ok());
    }
}
