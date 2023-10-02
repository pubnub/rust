//! Publish module.
//!
//! Publish message to a channel.
//! The publish module contains the [`PublishMessageBuilder`] and
//! [`PublishMessageViaChannelBuilder`]. The [`PublishMessageBuilder`] is used
//! to publish a message to a channel.
//!
//! This module is accountable for publishing a message to a channel of the
//! [`PubNub`] network.
//!
//! [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder]
//! [`PublishMessageViaChannelBuilder`]: crate::dx::publish::PublishMessageViaChannelBuilder]
//! [`PubNub`]:https://www.pubnub.com/

#[doc(inline)]
pub use result::{PublishResponseBody, PublishResult};
pub mod result;

#[doc(inline)]
pub use builders::{
    PublishMessageBuilder, PublishMessageViaChannel, PublishMessageViaChannelBuilder,
};
pub mod builders;

use crate::{
    core::{
        utils::{
            encoding::{url_encode, url_encode_extended, UrlEncodeExtension},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        CryptoProvider, Deserializer, PubNubError, Serialize, Transport, TransportMethod,
        TransportRequest,
    },
    dx::pubnub_client::{PubNubClientInstance, PubNubConfig},
    lib::{
        alloc::{
            format,
            string::{String, ToString},
            sync::Arc,
        },
        collections::HashMap,
        core::ops::Not,
    },
};

use base64::{engine::general_purpose, Engine as _};

impl<T, D> PubNubClientInstance<T, D>
where
    D: Deserializer,
{
    /// Create a new publish message builder.
    /// This method is used to publish a message to a channel.
    ///
    /// Instance of [`PublishMessageBuilder`] is returned.
    ///
    /// # Example
    /// ```
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
    /// #      })
    /// #     .with_user_id("uuid")
    /// #     .build()?;
    ///
    /// pubnub.publish_message("Hello, world!")
    ///    .channel("my_channel")
    ///    .execute()
    ///    .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder
    pub fn publish_message<M>(&self, message: M) -> PublishMessageBuilder<T, M, D>
    where
        M: Serialize,
    {
        let seqn = self.seqn();
        PublishMessageBuilder {
            message,
            pub_nub_client: self.clone(),
            seqn,
        }
    }

    fn seqn(&self) -> u16 {
        let mut locked_value = self.next_seqn.lock();
        let ret = *locked_value;

        if *locked_value == u16::MAX {
            *locked_value = 0;
        }
        *locked_value += 1;

        ret
    }
}

impl<T, M, D> PublishMessageViaChannelBuilder<T, M, D>
where
    M: Serialize,
    D: Deserializer,
{
    fn prepare_context_with_request(
        self,
    ) -> Result<PublishMessageContext<T, D, TransportRequest>, PubNubError> {
        let instance = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        PublishMessageContext::from(instance)
            .map_data(|client, params| {
                params.create_transport_request(&client.config, &client.cryptor.clone())
            })
            .map(|ctx| {
                Ok(PublishMessageContext {
                    client: ctx.client,
                    data: ctx.data?,
                })
            })
    }
}

impl<T, M, D> PublishMessageViaChannelBuilder<T, M, D>
where
    T: Transport,
    M: Serialize,
    D: Deserializer + 'static,
{
    /// Execute the request and return the result.
    /// This method is asynchronous and will return a future.
    /// The future will resolve to a [`PublishResult`] or [`PubNubError`].
    ///
    /// # Example
    /// ```no_run
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
    /// #      })
    /// #     .with_user_id("uuid")
    /// #     .build()?;
    ///
    /// pubnub.publish_message("Hello, world!")
    ///    .channel("my_channel")
    ///    .execute()
    ///    .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PublishResult`]: struct.PublishResult.html
    /// [`PubNubError`]: enum.PubNubError.html
    pub async fn execute(self) -> Result<PublishResult, PubNubError> {
        self.prepare_context_with_request()?
            .map(|some| async move {
                let deserializer = some.client.deserializer.clone();
                some.data
                    .send::<PublishResponseBody, _, _, _>(&some.client.transport, deserializer)
                    .await
            })
            .await
    }
}

#[cfg(feature = "blocking")]
impl<T, M, D> PublishMessageViaChannelBuilder<T, M, D>
where
    T: crate::core::blocking::Transport,
    M: Serialize,
    D: Deserializer + 'static,
{
    /// Execute the request and return the result.
    /// This method is asynchronous and will return a future.
    /// The future will resolve to a [`PublishResult`] or [`PubNubError`].
    ///
    /// # Example
    /// ```no_run
    /// # use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// # PubNubClientBuilder::with_reqwest_blocking_transport()
    /// #     .with_keyset(Keyset{
    /// #         subscribe_key: "demo",
    /// #         publish_key: Some("demo"),
    /// #         secret_key: None,
    /// #      })
    /// #     .with_user_id("uuid")
    /// #     .build()?;
    ///
    /// pubnub.publish_message("Hello, world!")
    ///    .channel("my_channel")
    ///    .execute_blocking()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PublishResult`]: struct.PublishResult.html
    /// [`PubNubError`]: enum.PubNubError.html
    pub fn execute_blocking(self) -> Result<PublishResult, PubNubError> {
        self.prepare_context_with_request()?
            .map_data(|client, request| {
                let client = client.clone();
                let deserializer = client.deserializer.clone();
                request
                    .send_blocking::<PublishResponseBody, _, _, _>(&client.transport, deserializer)
            })
            .data
    }
}

impl<M> PublishMessageParams<M>
where
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

        if let Some(r#type) = &self.r#type {
            query_params.insert("type".to_string(), r#type.clone());
        }

        query_params.insert("seqn".to_string(), self.seqn.to_string());

        self.meta
            .as_ref()
            .map(serialize_meta)
            .and_then(|meta| query_params.insert("meta".to_string(), meta));

        query_params
    }

    fn create_transport_request(
        self,
        config: &PubNubConfig,
        cryptor: &Option<Arc<dyn CryptoProvider + Send + Sync>>,
    ) -> Result<TransportRequest, PubNubError> {
        let query_params = self.prepare_publish_query_params();

        let pub_key = config
            .publish_key
            .as_ref()
            .ok_or_else(|| PubNubError::general_api_error("Publish key is not set", None, None))?;
        let sub_key = &config.subscribe_key;

        let mut m_vec = self.message.serialize()?;
        if let Some(cryptor) = cryptor {
            if let Ok(encrypted) = cryptor.encrypt(m_vec.to_vec()) {
                m_vec = format!("\"{}\"", general_purpose::STANDARD.encode(encrypted)).into_bytes();
            }
        }

        if self.use_post {
            Ok(TransportRequest {
                path: format!(
                    "/publish/{pub_key}/{sub_key}/0/{}/0",
                    url_encode(self.channel.as_bytes())
                ),
                method: TransportMethod::Post,
                query_parameters: query_params,
                body: Some(m_vec),
                headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            })
        } else {
            String::from_utf8(m_vec)
                .map_err(|e| PubNubError::Serialization {
                    details: e.to_string(),
                })
                .map(|m_str| TransportRequest {
                    path: format!(
                        "/publish/{}/{}/0/{}/0/{}",
                        pub_key,
                        sub_key,
                        url_encode(self.channel.as_bytes()),
                        url_encode_extended(m_str.as_bytes(), UrlEncodeExtension::NonChannelPath)
                    ),
                    method: TransportMethod::Get,
                    query_parameters: query_params,
                    ..Default::default()
                })
        }
    }
}

struct PublishMessageContext<T, D, X> {
    client: PubNubClientInstance<T, D>,
    data: X,
}

impl<T, D, M> From<PublishMessageViaChannel<T, M, D>>
    for PublishMessageContext<T, D, PublishMessageParams<M>>
where
    M: Serialize,
    D: Deserializer,
{
    fn from(value: PublishMessageViaChannel<T, M, D>) -> Self {
        Self {
            client: value.pub_nub_client,
            data: PublishMessageParams {
                channel: value.channel,
                message: value.message,
                store: value.store,
                ttl: value.ttl,
                meta: value.meta,
                seqn: value.seqn,
                replicate: value.replicate,
                use_post: value.use_post,
                space_id: value.space_id,
                r#type: value.r#type,
            },
        }
    }
}

impl<T, D, X> PublishMessageContext<T, D, X> {
    fn map_data<F, Y>(self, f: F) -> PublishMessageContext<T, D, Y>
    where
        F: FnOnce(PubNubClientInstance<T, D>, X) -> Y,
    {
        let client = self.client;
        let data = f(client.clone(), self.data);

        PublishMessageContext { client, data }
    }

    fn map<F, Y>(self, f: F) -> Y
    where
        F: FnOnce(Self) -> Y,
    {
        f(self)
    }
}

struct PublishMessageParams<M> {
    message: M,
    seqn: u16,
    channel: String,
    store: Option<bool>,
    replicate: bool,
    ttl: Option<u32>,
    use_post: bool,
    meta: Option<HashMap<String, String>>,
    space_id: Option<String>,
    r#type: Option<String>,
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

#[cfg(test)]
mod should {
    use super::*;
    use crate::providers::deserialization_serde::DeserializerSerde;
    use crate::{
        core::TransportResponse,
        dx::pubnub_client::{PubNubClientInstance, PubNubClientRef, PubNubConfig},
        lib::{
            alloc::{sync::Arc, vec},
            collections::HashMap,
        },
        transport::middleware::PubNubMiddleware,
        Keyset, PubNubClientBuilder,
    };
    use test_case::test_case;

    #[derive(Default, Debug)]
    struct MockTransport;

    fn client() -> PubNubClientInstance<PubNubMiddleware<MockTransport>, DeserializerSerde> {
        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(
                &self,
                _request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                Ok(TransportResponse {
                    status: 200,
                    body: Some(b"[1, \"Sent\", \"1234567890\"]".to_vec()),
                    ..Default::default()
                })
            }
        }

        PubNubClientBuilder::with_transport(MockTransport)
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
        #[derive(Default, Clone)]
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

        let client = client();

        let result = client
            .publish_message("First message")
            .channel("IGuess")
            .replicate(true)
            .execute()
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn verify_all_query_parameters() {
        let client = client();

        let result = client
            .publish_message("message")
            .channel("chan")
            .replicate(false)
            .ttl(50)
            .store(true)
            .space_id("space_id")
            .r#type("message_type")
            .meta(HashMap::from([("k".to_string(), "v".to_string())]))
            .prepare_context_with_request()
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
            result.data.query_parameters
        );
    }

    #[test]
    fn verify_seqn_is_incrementing() {
        let client = client();

        let received_sequence_numbers = vec![
            client.publish_message("message").seqn,
            client.publish_message("message").seqn,
        ];

        assert_eq!(vec![1, 2], received_sequence_numbers);
    }

    #[tokio::test]
    async fn return_err_if_publish_key_is_not_provided() {
        let client = {
            let default_client = client();
            let ref_client = Arc::try_unwrap(default_client.inner).unwrap();

            PubNubClientInstance {
                inner: Arc::new(PubNubClientRef {
                    config: PubNubConfig {
                        publish_key: None,
                        ..ref_client.config
                    },
                    ..ref_client
                }),
            }
        };

        assert!(client
            .publish_message("message")
            .channel("chan")
            .execute()
            .await
            .is_err());
    }

    #[test]
    fn test_send_string_when_get() {
        let client = client();
        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .prepare_context_with_request()
            .unwrap();

        assert_eq!(
            format!(
                "/publish///0/{}/0/{}",
                channel,
                url_encode_extended(
                    format!("\"{}\"", message).as_bytes(),
                    UrlEncodeExtension::NonChannelPath
                )
            ),
            result.data.path
        );
    }

    #[test]
    fn test_send_map_when_get() {
        let client = client();
        let channel = String::from("ch");
        let message = HashMap::from([("a", "b")]);

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .prepare_context_with_request()
            .unwrap();

        assert_eq!(
            format!(
                "/publish///0/{}/0/{}",
                channel,
                url_encode_extended(
                    "{\"a\":\"b\"}".as_bytes(),
                    UrlEncodeExtension::NonChannelPath
                )
            ),
            result.data.path
        );
    }

    #[test]
    fn test_quotes_not_escaped_when_post() {
        let client = client();
        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .use_post(true)
            .prepare_context_with_request()
            .unwrap();

        let result_data = result.data;
        assert_eq!(format!("/publish///0/{}/0", channel), result_data.path);
        assert_eq!(
            format!("\"{}\"", message),
            String::from_utf8(result_data.body.unwrap()).unwrap()
        );
    }

    #[test]
    fn test_path_segments_get() {
        let client = client();
        let channel = String::from("channel_name");
        let message = HashMap::from([("number", 7)]);

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .prepare_context_with_request()
            .unwrap();

        assert_eq!(
            format!(
                "/publish///0/{}/0/{}",
                channel,
                url_encode_extended(
                    "{\"number\":7}".as_bytes(),
                    UrlEncodeExtension::NonChannelPath
                )
            ),
            result.data.path
        );
    }

    #[test]
    fn test_path_segments_post() {
        let client = client();
        let channel = String::from("channel_name");
        let message = HashMap::from([("number", 7)]);

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .use_post(true)
            .prepare_context_with_request()
            .unwrap();
        assert_eq!(format!("/publish///0/{}/0", channel), result.data.path);
    }

    #[test_case(HashMap::from([("k".to_string(), "v".to_string())]), "{\"k\":\"v\"}" ; "hash map with elements")]
    #[test_case(HashMap::new(), "{}" ; "empty hash map")]
    #[test_case(HashMap::from([("k".to_string(), "".to_string())]), "{\"k\":\"\"}" ; "empty value")]
    #[test_case(HashMap::from([("".to_string(), "v".to_string())]), "{\"\":\"v\"}" ; "empty key")]
    fn this_test_should_test_an_fn_itself(map: HashMap<String, String>, expected_json: &str) {
        let result = serialize_meta(&map);

        assert_eq!(expected_json, result);
    }

    #[tokio::test]
    async fn return_error_for_error_response() {
        #[derive(Default)]
        struct MockTransport;

        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(
                &self,
                _request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                Ok(TransportResponse {
                    status: 400,
                    body: Some("{\"error\":true,\"message\":\"error message\"}".into()),
                    ..Default::default()
                })
            }
        }

        let client = PubNubClientBuilder::with_transport(MockTransport)
            .with_keyset(Keyset {
                publish_key: Some(""),
                subscribe_key: "",
                secret_key: None,
            })
            .with_user_id("user_id")
            .build()
            .unwrap();

        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .execute()
            .await;

        assert!(result.is_err());
    }
}
