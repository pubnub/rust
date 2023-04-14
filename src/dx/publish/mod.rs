//! Publish module.
//!
//! Publish message to a channel.
//! The publish module contains the [`PublishMessageBuilder`] and [`PublishMessageViaChannelBuilder`].
//! The [`PublishMessageBuilder`] is used to publish a message to a channel.
//!
//! This module is accountable for publishing a message to a channel of the [`PubNub`] network.
//!
//! [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder]
//! [`PublishMessageViaChannelBuilder`]: crate::dx::publish::PublishMessageViaChannelBuilder]
//! [`PubNub`]:https://www.pubnub.com/

#[doc(inline)]
pub use result::{PublishResponseBody, PublishResult};
pub mod result;

#[doc(inline)]
pub use builders::PublishMessageBuilder;
pub mod builders;

use crate::{
    core::{
        headers::{APPLICATION_JSON, CONTENT_TYPE},
        Deserializer, PubNubError, Serialize, Transport, TransportMethod, TransportRequest,
        TransportResponse,
    },
    dx::PubNubClient,
};
use builders::{PublishMessageViaChannel, PublishMessageViaChannelBuilder};
use result::body_to_result;
use std::{collections::HashMap, ops::Not};
use urlencoding::encode;

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// Create a new publish message builder.
    /// This method is used to publish a message to a channel.
    ///
    /// Instance of [`PublishMessageBuilder`] is returned.
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
    /// [`PublishMessageBuilder`]: crate::dx::publish::PublishMessageBuilder]
    pub fn publish_message<M>(&self, message: M) -> PublishMessageBuilder<T, M>
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
        let mut locked_value = self
            .next_seqn
            .lock()
            .expect("dx::publish seqn lock poisoned!");
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
    T: Transport,
    M: Serialize,
    D: for<'de> Deserializer<'de, PublishResponseBody>,
{
    /// Execute the request and return the result.
    /// This method is asynchronous and will return a future.
    /// The future will resolve to a [`PublishResponse`] or [`PubNubError`].
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
    /// [`PublishResponse`]: struct.PublishResponse.html
    /// [`PubNubError`]: enum.PubNubError.html
    pub async fn execute(self) -> Result<PublishResult, PubNubError> {
        let instance = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None))?;

        let client: PubNubClient<_> = instance.pub_nub_client.clone();

        // TODO: ref: builders.rs[1]
        let deserializer = instance.deserializer.clone();

        instance
            .create_transport_request()
            .map(|request| async move { Self::send_request(&client.transport, request).await })?
            .await
            .map(|response| Self::response_to_result(&deserializer, response))?
    }

    async fn send_request(
        transport: &T,
        request: TransportRequest,
    ) -> Result<TransportResponse, PubNubError> {
        transport.send(request).await
    }

    // TODO: Maybe it will be possible to extract this into a middleware.
    //       Currently, it's not necessary, but it might be very useful
    //       to not have to do it manually in each dx module.
    fn response_to_result(
        deserializer: &D,
        response: TransportResponse,
    ) -> Result<PublishResult, PubNubError> {
        response
            .body
            .map(|body| deserializer.deserialize(&body))
            .transpose()
            .and_then(|body| {
                body.ok_or_else(|| {
                    PubNubError::general_api_error(
                        format!("No body in the response! Status code: {}", response.status),
                        Some(response.status),
                    )
                })
                .map(|body| body_to_result(body, response.status))
            })?
    }
}

impl<T, M, D> PublishMessageViaChannel<T, M, D>
where
    T: Transport,
    M: Serialize,
    D: for<'de> Deserializer<'de, PublishResponseBody>,
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

    // TODO: create test for path creation!
    fn create_transport_request(self) -> Result<TransportRequest, PubNubError> {
        let query_params = self.prepare_publish_query_params();

        let pub_key = &self
            .pub_nub_client
            .config
            .publish_key
            .as_ref()
            .ok_or_else(|| PubNubError::general_api_error("Publish key is not set", None))?;
        let sub_key = &self.pub_nub_client.config.subscribe_key;

        if self.use_post {
            self.message.serialize().map(|m_vec| TransportRequest {
                path: format!("/publish/{pub_key}/{sub_key}/0/{}/0", encode(&self.channel)),
                method: TransportMethod::Post,
                query_parameters: query_params,
                body: Some(m_vec),
                headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            })
        } else {
            self.message
                .serialize()
                .and_then(|m_vec| {
                    String::from_utf8(m_vec).map_err(|e| PubNubError::Serialization(e.to_string()))
                })
                .map(|m_str| TransportRequest {
                    path: format!(
                        "/publish/{}/{}/0/{}/0/{}",
                        pub_key,
                        sub_key,
                        encode(&self.channel),
                        encode(&m_str)
                    ),
                    method: TransportMethod::Get,
                    query_parameters: query_params,
                    ..Default::default()
                })
        }
    }
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
    use std::sync::Arc;

    use super::*;
    use crate::{
        core::TransportResponse,
        dx::{
            pubnub_client::{PubNubClientRef, PubNubConfig},
            PubNubClient,
        },
        transport::middleware::PubNubMiddleware,
        Keyset, PubNubClientBuilder,
    };
    use test_case::test_case;

    #[derive(Default, Debug)]
    struct MockTransport;

    fn client() -> PubNubClient<PubNubMiddleware<MockTransport>> {
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
            .channel("Iguess")
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

    #[test]
    fn verify_seqn_is_incrementing() {
        let client = client();

        let received_seqns = vec![
            client.publish_message("meess").seqn,
            client.publish_message("meess").seqn,
        ];

        assert_eq!(vec![1, 2], received_seqns);
    }

    #[tokio::test]
    async fn return_err_if_publish_key_is_not_provided() {
        let client = {
            let default_client = client();

            let ref_client = Arc::try_unwrap(default_client.inner).unwrap();

            PubNubClient {
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
            .publish_message("meess")
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
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(
            format!(
                "/publish///0/{}/0/{}",
                channel,
                encode(&format!("\"{}\"", message))
            ),
            result.path
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
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(
            format!("/publish///0/{}/0/{}", channel, encode("{\"a\":\"b\"}")),
            result.path
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
            .build()
            .unwrap()
            .create_transport_request()
            .unwrap();

        assert_eq!(format!("/publish///0/{}/0", channel), result.path);
        assert_eq!(
            format!("\"{}\"", message),
            String::from_utf8(result.body.unwrap()).unwrap()
        );
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

        let client = PubNubClientBuilder::<MockTransport>::new()
            .with_transport(MockTransport::default())
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
