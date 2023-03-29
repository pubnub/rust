//! TODO: split into submodules
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

use crate::core::headers::{APPLICATION_JSON, CONTENT_TYPE};
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;
use crate::{
    core::{
        Deserializer, PubNubError, Serialize, Transport, TransportMethod, TransportRequest,
        TransportResponse,
    },
    dx::PubNubClient,
};
use derive_builder::Builder;
use std::ops::Not;
use std::{collections::HashMap, rc::Rc};
use urlencoding::encode;

/// The [`PublishMessageBuilder`] is used to publish a message to a channel.
///
/// This struct is used to publish a message to a channel. It is used by the [`publish_message`] method of the [`PubNubClient`].
/// The [`publish_message`] method is used to publish a message to a channel.
/// The [`PublishMessageBuilder`] is used to build the request to be sent to the [`PubNub`] network.
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
    /// The [`channel`] method is used to set the channel to publish the message to.
    ///
    /// [`channel`]: crate::dx::publish::PublishMessageBuilder::channel
    #[cfg(feature = "serde")]
    pub fn channel<S>(
        self,
        channel: S,
    ) -> PublishMessageViaChannelBuilder<'pub_nub, T, M, SerdeDeserializer>
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

    #[cfg(not(feature = "serde"))]
    pub fn channel<S>(self, channel: S) -> PublishMessageDeserializerBuilder<'pub_nub, T, M>
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

/// The [`PublishMessageDeserializer`] is used to publish a message to a channel.
///
/// This struct is used to publish a message to a channel. It is used by the [`publish_message`] method of the [`PubNubClient`].
/// It adds the deserializer to the [`PublishMessageBuilder`].
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
pub struct PublishMessageDeserializerBuilder<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    pub_nub_client: &'pub_nub PubNubClient<T>,
    message: M,
    seqn: u16,
    channel: String,
}

#[cfg(not(feature = "serde"))]
impl<'pub_nub, T, M> PublishMessageDeserializerBuilder<'pub_nub, T, M>
where
    T: Transport,
    M: Serialize,
{
    /// The [`deserialize_with`] method is used to set the deserializer to use to deserialize the response.
    /// It's important to note that the deserializer must implement the [`Deserializer`] trait for
    /// the [`PublishResponse`] type.
    ///
    /// [`deserialize_with`]: crate::dx::publish::PublishMessageDeserializerBuilder::deserialize_with
    /// [`Deserializer`]: crate::core::Deserializer
    /// [`PublishResponse`]: crate::core::publish::PublishResponse
    pub fn deserialize_with<D>(
        self,
        deserializer: D,
    ) -> PublishMessageViaChannelBuilder<'pub_nub, T, M, D>
    where
        for<'de> D: Deserializer<'de, PublishResponseBody>,
    {
        PublishMessageViaChannelBuilder {
            pub_nub_client: Some(self.pub_nub_client),
            seqn: Some(self.seqn),
            deserializer: Some(Rc::new(deserializer)),
            ..Default::default()
        }
        .message(self.message)
        .channel(self.channel)
    }
}

/// The [`PublishMessageViaChannelBuilder`] is used to publish a message to a channel.
/// This struct is used to publish a message to a channel. It is used by the [`publish_message`] method of the [`PubNubClient`].
///
/// This is next step in the publish process. The [`PublishMessageViaChannelBuilder`] is used to build the request to be sent to the [`PubNub`] network.
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
/// [`PublishMessageViaChannelBuilder`]: crate::dx::publish::PublishMessageViaChannelBuilder
/// [`publish_message`]: crate::dx::PubNubClient::publish_message
/// [`PubNub`]:https://www.pubnub.com/
/// [`PubNubClient`]: crate::dx::PubNubClient
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(private))]
pub struct PublishMessageViaChannel<'pub_nub, T, M, D>
where
    T: Transport,
    M: Serialize,
    D: for<'de> Deserializer<'de, PublishResponseBody>,
{
    #[builder(setter(custom))]
    pub_nub_client: &'pub_nub PubNubClient<T>,

    #[builder(setter(custom))]
    seqn: u16,

    // TODO: Moved to heap to avoid partial move, but this is not ideal. ref[1]
    #[builder(setter(custom))]
    deserializer: Rc<D>,

    /// Message to publish
    message: M,

    /// Channel to publish to
    #[builder(setter(into))]
    channel: String,

    /// Switch that decide if the message should be stored in history
    #[builder(setter(strip_option), default = "None")]
    store: Option<bool>,

    /// Switch that decide if the transaction should be replicated
    /// following the PubNub replication rules.
    ///
    /// See more at [`PubNub replication rules`]
    ///
    /// [`PubNub replication rules`]:https://www.pubnub.com/pricing/transaction-classification/
    #[builder(default = "true")]
    replicate: bool,

    /// Set a per-message TTL time to live in storage.
    #[builder(setter(strip_option), default = "None")]
    ttl: Option<u32>,

    /// Switch that decide if the message should be published using POST method.
    #[builder(setter(strip_option), default = "false")]
    use_post: bool,

    /// Object to send additional information about the message.
    #[builder(setter(strip_option), default = "None")]
    meta: Option<HashMap<String, String>>,

    /// Space ID to publish to.
    #[builder(setter(strip_option), default = "None")]
    space_id: Option<String>,

    /// Message type to publish.
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

impl<'pub_nub, T, M, D> PublishMessageViaChannel<'pub_nub, T, M, D>
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

    // TODO: create test for path creation!
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
                path: format!("publish/{pub_key}/{sub_key}/0/{}/0", encode(&self.channel)),
                method: TransportMethod::Post,
                query_parameters: query_params,
                body: Some(m_vec),
                headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
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

impl<'pub_nub, T, M, D> PublishMessageViaChannelBuilder<'pub_nub, T, M, D>
where
    T: Transport,
    M: Serialize,
    D: for<'de> Deserializer<'de, PublishResponseBody>,
{
    /// Deserializer to use to deserialize the response.
    /// It's important to note that the deserializer must implement the [`Deserializer`] trait for
    /// the [`PublishResponse`] type.
    pub fn deserialize_with<D2>(
        self,
        deserializer: D2,
    ) -> PublishMessageViaChannelBuilder<'pub_nub, T, M, D2>
    where
        D2: for<'de> Deserializer<'de, PublishResponseBody>,
    {
        PublishMessageViaChannelBuilder {
            pub_nub_client: self.pub_nub_client,
            channel: self.channel,
            message: self.message,
            seqn: self.seqn,
            store: self.store,
            replicate: self.replicate,
            ttl: self.ttl,
            use_post: self.use_post,
            meta: self.meta,
            space_id: self.space_id,
            message_type: self.message_type,
            deserializer: Some(Rc::new(deserializer)),
        }
    }

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
            .map_err(|err| PubNubError::PublishError(err.to_string()))?;

        let client = instance.pub_nub_client;

        // TODO: [1]
        let deserializer = instance.deserializer.clone();

        instance
            .create_transport_request()
            .map(|request| Self::send_request(&client.transport, request))?
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
                    PubNubError::PublishError(format!(
                        "No body in the response! Status code: {}",
                        response.status
                    ))
                })
                .map(|body| match body {
                    PublishResponseBody::PublishResponse(error_indicator, message, timetoken) => {
                        if error_indicator == 1 {
                            Ok(PublishResult { timetoken })
                        } else {
                            Err(PubNubError::PublishError(message))
                        }
                    }
                    PublishResponseBody::OtherResponse(body) => Err(PubNubError::PublishError(
                        format!("Status code: {}, body: {:?}", response.status, body),
                    )),
                })
            })?
    }
}

/// TODO: docs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishResult {
    pub timetoken: String,
}

/// TODO: docs
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PublishResponseBody {
    /// TODO: docs
    PublishResponse(i32, String, String),
    /// TODO: docs
    OtherResponse(OtherResponse),
}

/// TODO: docs
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OtherResponse {
    /// TODO: docs
    pub status: u16,

    /// TODO: docs
    pub error: bool,

    /// TODO: docs
    pub service: String,

    /// TODO: docs
    pub message: String,
}

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

    /// Create a new publish message builder.
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
        transport::middleware::PubNubMiddleware,
        Keyset, PubNubClientBuilder,
    };
    use test_case::test_case;

    #[derive(Default)]
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
            .channel("Iguess")
            .replicate(true)
            .execute()
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn verify_all_query_parameters() {
        let mut client = client();

        let result = client
            .publish_message("message")
            .channel("chan")
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

    #[test]
    fn verify_seqn_is_incrementing() {
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
            .channel("chan")
            .execute()
            .await
            .is_err());
    }

    #[test]
    fn test_send_string_when_get() {
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
            format!(
                "publish///0/{}/0/{}",
                channel,
                encode(&format!("\"{}\"", message))
            ),
            result.path
        );
    }

    #[test]
    fn test_send_map_when_get() {
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
            format!("publish///0/{}/0/{}", channel, encode("{\"a\":\"b\"}")),
            result.path
        );
    }

    #[test]
    fn test_quotes_not_escaped_when_post() {
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
    fn this_test_should_test_an_fn_itself(map: HashMap<String, String>, expected_json: &str) {
        let result = serialize_meta(&map);

        assert_eq!(expected_json, result);
    }

    #[tokio::test]
    async fn deserialize_response() {
        let mut client = client();

        let channel = String::from("ch");
        let message = "this is message";

        let result = client
            .publish_message(message)
            .channel(channel.clone())
            .execute()
            .await
            .unwrap();

        assert_eq!(
            result,
            PublishResult {
                timetoken: "1234567890".into()
            }
        );
    }

    #[tokio::test]
    async fn deserialize_response_with_error() {
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

        let mut client = PubNubClientBuilder::<MockTransport>::new()
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
