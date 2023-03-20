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
    seqn: u16,
}

impl<'pub_nub, T> PublishMessageBuilder<'pub_nub, T>
where
    T: Transport,
{
    /// TODO: Add documentation
    pub fn channel(self, channel: String) -> PublishMessageViaChannelBuilder<'pub_nub, T> {
        PublishMessageViaChannelBuilder {
            pub_nub_client: Some(self.pub_nub_client),
            seqn: Some(self.seqn),
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
    #[builder(setter(custom))]
    seqn: u16,
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

impl<'pub_nub, T> PublishMessageViaChannel<'pub_nub, T>
where
    T: Transport,
{
    fn prepare_publish_query_params(&self) -> HashMap<String, String> {
        let mut query_params: HashMap<String, String> = HashMap::new();

        self.store
            .and_then(|s| query_params.insert("store".to_string(), bool_to_numeric(s)));

        self.ttl
            .and_then(|t| query_params.insert("ttl".to_string(), t.to_string()));

        if !self.replicate {
            query_params.insert("norep".to_string(), true.to_string());
        }

        if let Some(space_id) = &self.space_id {
            query_params.insert("space-id".to_string(), space_id.clone());
        }

        if let Some(message_type) = &self.message_type {
            query_params.insert("type".to_string(), message_type.clone());
        }

        query_params.insert("seqn".to_string(), self.seqn.to_string());

        query_params
    }

    fn to_transport_request(&self) -> TransportRequest {
        let query_params = self.prepare_publish_query_params();
        let pub_key = "";
        let sub_key = "";

        if self.use_post {
            TransportRequest {
                path: format!("publish/{sub_key}/{pub_key}/0/{}/0", self.channel),
                method: TransportMethod::Post,
                query_parameters: query_params,
                //body: self.message.unwrap(), TODO
                ..Default::default()
            }
        } else {
            TransportRequest {
                path: format!(
                    "publish/{}/{}/0/{}/0/\"{}\"",
                    sub_key, pub_key, self.channel, self.message
                ),
                method: TransportMethod::Get,
                query_parameters: query_params,
                ..Default::default()
            }
        }
    }
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

        let request = instance.to_transport_request();

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
    fn seqn(&mut self) -> u16 {
        let ret = self.next_seqn;
        if self.next_seqn == u16::MAX {
            self.next_seqn = 0;
        }
        self.next_seqn += 1;
        ret
    }

    /// TODO: Add documentation
    pub fn publish_message(&mut self, message: MessageType) -> PublishMessageBuilder<T> {
        let seqn = self.seqn();
        PublishMessageBuilder {
            message,
            pub_nub_client: self,
            seqn,
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::TransportResponse,
        dx::{pubnub_client::PubNubConfig, PubNubClient},
        Keyset,
    };

    #[derive(Default)]
    struct MockTransport;

    fn client() -> PubNubClient<MockTransport> {
        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(
                &self,
                _request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                Ok(TransportResponse::default())
            }
        }

        PubNubClient::with_transport(MockTransport::default())
            .with_keyset(Keyset {
                publish_key: Some(""),
                subscribe_key: "",
                secret_key: None,
            })
            .with_user_id("")
            .build()
            .unwrap()
    }

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

        let mut client = client();

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
        let mut client = client();

        let result = client
            .publish_message("message".into())
            .channel("chan".into())
            .replicate(false)
            .ttl(50)
            .store(true)
            .space_id("space_id".into())
            .message_type("message_type".into())
            .build()
            .unwrap()
            .to_transport_request();

        assert_eq!(
            HashMap::<String, String>::from([
                ("norep".into(), "true".into()),
                ("store".into(), "1".into()),
                ("space-id".into(), "space_id".into()),
                ("type".into(), "message_type".into()),
                ("ttl".into(), "50".into()),
                ("seqn".into(), "1".into())
            ]),
            result.query_parameters
        );
    }

    #[tokio::test]
    async fn verify_seqn_is_incrementing() {
        let mut client = client();

        let received_seqns = vec![
            client.publish_message("meess".into()).seqn,
            client.publish_message("meess".into()).seqn,
        ];

        assert_eq!(vec![1, 2], received_seqns);
    }
}
