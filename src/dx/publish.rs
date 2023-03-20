//! TODO: Add documentation
use crate::{
    core::{PubNubError, Transport, TransportMethod, TransportRequest},
    dx::PubNubClient,
    Serialize,
};
use derive_builder::Builder;
use std::collections::HashMap;
use std::ops::Not;

/// TODO: Add documentation
pub type MessageType = String;

/// TODO: Add documentation
pub struct PublishMessageBuilder<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    pub_nub_client: &'pub_nub PubNubClient<T>,
    message: M,
    seqn: u16,
}

impl<'pub_nub, T, M> PublishMessageBuilder<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    /// TODO: Add documentation
    pub fn channel(self, channel: String) -> PublishMessageViaChannelBuilder<'pub_nub, T, M> {
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
pub struct PublishMessageViaChannel<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    #[builder(setter(custom))]
    pub_nub_client: &'pub_nub PubNubClient<T>,
    #[builder(setter(custom))]
    seqn: u16,
    /// TODO: Add documentation
    message: M,
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

fn serialize_meta(meta: &HashMap<String, String>) -> String {
    let mut result = String::new();
    result.push('{');
    meta.iter().for_each(|k| {
        result.push_str(format!("\"{}\":\"{}\",", k.0.as_str(), k.1.as_str()).as_str());
    });
    if result.ends_with(',') {
        result.remove(result.len() - 1);
    }
    result.push('}');
    result
}

impl<'pub_nub, T, M> PublishMessageViaChannel<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    fn prepare_publish_query_params(&self) -> HashMap<String, String> {
        let mut query_params: HashMap<String, String> = HashMap::new();

        self.store
            .and_then(|s| query_params.insert("store".to_string(), bool_to_numeric(s)));

        self.ttl
            .and_then(|t| query_params.insert("ttl".to_string(), t.to_string()));

        self.replicate
            .not()
            .then(|| query_params.insert("norep".to_string(), true.to_string()));

        if let Some(space_id) = &self.space_id {
            query_params.insert("space-id".to_string(), space_id.clone());
        }

        if let Some(message_type) = &self.message_type {
            query_params.insert("type".to_string(), message_type.clone());
        }

        query_params.insert("seqn".to_string(), self.seqn.to_string());

        self.meta
            .as_ref()
            .map(serialize_meta)
            .and_then(|meta| query_params.insert("meta".to_string(), meta));

        query_params
    }

    fn create_transport_request(self) -> Result<TransportRequest, PubNubError> {
        let query_params = self.prepare_publish_query_params();

        let pub_key = &self
            .pub_nub_client
            .config
            .publish_key
            .as_ref()
            .ok_or_else(|| PubNubError::PublishError("Publish key is not set".into()))?;
        let sub_key = &self.pub_nub_client.config.subscribe_key;

        if self.use_post {
            self.message.serialize().map(|m_vec| TransportRequest {
                path: format!("publish/{sub_key}/{pub_key}/0/{}/0", self.channel),
                method: TransportMethod::Post,
                query_parameters: query_params,
                body: Some(m_vec),
                ..Default::default()
            })
        } else {
            self.message
                .serialize()
                .and_then(|m_vec| {
                    String::from_utf8(m_vec)
                        .map_err(|e| PubNubError::SerializationError(e.to_string()))
                })
                .map(|m_str| TransportRequest {
                    path: format!(
                        "publish/{}/{}/0/{}/0/{}",
                        sub_key, pub_key, self.channel, m_str
                    ),
                    method: TransportMethod::Get,
                    query_parameters: query_params,
                    ..Default::default()
                })
        }
    }
}

impl<'pub_nub, T, M> PublishMessageViaChannelBuilder<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    /// TODO: Add documentation
    pub async fn execute(self) -> Result<PublishResult, PubNubError> {
        let instance = self
            .build()
            .map_err(|err| PubNubError::PublishError(err.to_string()))?;

        let client = instance.pub_nub_client;
        match instance.create_transport_request() {
            Ok(request) => client.transport.send(request).await.map(|_| PublishResult),
            Err(error) => Err(error),
        }
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
    pub fn publish_message<M>(&mut self, message: M) -> PublishMessageBuilder<T, M>
    where
        M: Serialize,
    {
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
    use test_case::test_case;

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
            .publish_message("First message")
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
            .publish_message("message")
            .channel("chan".into())
            .replicate(false)
            .ttl(50)
            .store(true)
            .space_id("space_id".into())
            .message_type("message_type".into())
            .meta(HashMap::from([("k".to_string(), "v".to_string())]))
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(
            HashMap::<String, String>::from([
                ("norep".into(), "true".into()),
                ("store".into(), "1".into()),
                ("space-id".into(), "space_id".into()),
                ("type".into(), "message_type".into()),
                ("meta".into(), "{\"k\":\"v\"}".into()),
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
            client.publish_message("meess").seqn,
            client.publish_message("meess").seqn,
        ];

        assert_eq!(vec![1, 2], received_seqns);
    }

    #[tokio::test]
    async fn return_err_if_publish_key_is_not_provided() {
        let mut client = {
            let default_client = client();

            PubNubClient {
                config: PubNubConfig {
                    publish_key: None,
                    ..default_client.config
                },
                ..default_client
            }
        };

        assert!(client
            .publish_message("meess")
            .channel("chan".into())
            .execute()
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_send_string_when_get() {
        let mut client = client();
        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(
            format!("publish///0/{}/0/\"{}\"", channel, message),
            result.path
        );
    }

    #[tokio::test]
    async fn test_send_map_when_get() {
        let mut client = client();
        let channel = String::from("ch");
        let message = HashMap::from([("a", "b")]);

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(
            format!("publish///0/{}/0/{{\"a\":\"b\"}}", channel),
            result.path
        );
    }

    #[tokio::test]
    async fn test_quotes_not_escaped_when_post() {
        let mut client = client();
        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .use_post(true)
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(format!("publish///0/{}/0", channel), result.path);
        assert_eq!(
            format!("\"{}\"", message),
            String::from_utf8(result.body.unwrap()).unwrap()
        );
    }

    #[test_case(HashMap::from([("k".to_string(), "v".to_string())]), "{\"k\":\"v\"}" ; "hash map with elements")]
    #[test_case(HashMap::new(), "{}" ; "empty hash map")]
    #[test_case(HashMap::from([("k".to_string(), "".to_string())]), "{\"k\":\"\"}" ; "empty value")]
    #[test_case(HashMap::from([("".to_string(), "v".to_string())]), "{\"\":\"v\"}" ; "empty key")]
    #[tokio::test]
    async fn this_test_should_test_an_fn_itself(map: HashMap<String, String>, expected_json: &str) {
        let result = serialize_meta(&map);

        assert_eq!(expected_json, result);
    }
}
