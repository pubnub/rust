//! Access Manager module.
//!
//! Manage resource endpoints access token.
//! This access manager module contains the [`GrantTokenRequestBuilder`],
//! [`RevokeTokenRequestBuilder`], [`ChannelPermission`],
//! [`ChannelGroupPermission`] and [`UserIdPermission`] which is used for access
//! token management.
//!
//! This module is accountable for granting and revoking access permissions to
//! resources of the [`PubNub`] network.
//!
//! [`PubNub`]:https://www.pubnub.com/

#[doc(inline)]
pub(crate) use payloads::*;
pub(crate) mod payloads;

#[doc(inline)]
pub use types::MetaValue;
pub mod types;

#[doc(inline)]
pub use result::{
    GrantTokenResponseBody, GrantTokenResult, RevokeTokenResponseBody, RevokeTokenResult,
};
pub mod result;

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
pub use permissions::*;
pub mod permissions;

#[cfg(feature = "serde")]
use crate::providers::{
    deserialization_serde::SerdeDeserializer, serialization_serde::SerdeSerializer,
};
use crate::{core::Transport, dx::PubNubClient};

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// Create grant token permissions request builder.
    /// This method is used to generate token with required permissions.
    ///
    /// Instance of [`GrantTokenRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    /// #    PubNubClientBuilder, Keyset,
    /// };
    /// # use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .grant_token(10)
    ///     .resources(&[permissions::channel("test-channel").read().write()])
    ///     .meta(HashMap::from([
    ///          ("role".into(), "administrator".into()),
    ///          ("access-duration".into(), 2800.into()),
    ///          ("ping-interval".into(), 1754.88.into()),
    ///      ]))
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(feature = "serde")]
    pub fn grant_token(
        &self,
        ttl: usize,
    ) -> GrantTokenRequestBuilder<T, SerdeSerializer, SerdeDeserializer> {
        GrantTokenRequestBuilder {
            pubnub_client: Some(self.clone()),
            serializer: Some(SerdeSerializer),
            deserializer: Some(SerdeDeserializer),
            ttl: Some(ttl),
            ..Default::default()
        }
    }
    /// Create grant token permissions request builder.
    /// This method is used to generate token with required permissions.
    ///
    /// Instance of [`GrantTokenRequestWithSerializerBuilder`] returned.
    ///
    /// # Example
    /// ```rust, no_run
    /// use pubnub::{
    ///     access::*,
    ///     core::{Deserializer, PubNubError, Serializer},
    /// #    PubNubClientBuilder, Keyset,
    /// };
    /// # use std::collections::HashMap;
    ///
    /// struct MySerializer;
    /// struct MyDeserializer;
    ///
    /// impl<'se> Serializer<'se, GrantTokenPayload> for MySerializer {
    ///    fn serialize(&self, object: &'se T) -> Result<Vec<u8>, PubNubError> {
    ///         // ...
    /// #        Ok(vec![])
    ///     }
    /// }
    ///
    /// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
    ///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
    ///         // ...
    /// #        Ok(GrantTokenResult { token: "<generated token>".into() })
    ///     }
    /// }
    ///
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .grant_token(10)
    ///     .serialize_with(MySerializer)
    ///     .derialize_with(MyDeserializer)
    ///     .resources(&[permissions::channel("test-channel").read().write()])
    ///     .meta(HashMap::from([
    ///          ("role".into(), "administrator".into()),
    ///          ("access-duration".into(), 2800.into()),
    ///          ("ping-interval".into(), 1754.88.into()),
    ///      ]))
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "serde"))]
    pub fn grant_token(&self, ttl: usize) -> GrantTokenRequestWithSerializerBuilder<T> {
        GrantTokenRequestWithSerializerBuilder {
            pubnub_client: self.clone(),
            ttl,
        }
    }

    /// Create grant token request builder.
    /// This method is used to revoke token permissions.
    ///
    /// Instance of [`RevokeTokenRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    /// #    PubNubClientBuilder, Keyset,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .revoke_token("p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI")
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(feature = "serde")]
    pub fn revoke_token<S>(&self, token: S) -> RevokeTokenRequestBuilder<T, SerdeDeserializer>
    where
        S: Into<String>,
    {
        RevokeTokenRequestBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(SerdeDeserializer),
            token: Some(token.into()),
        }
    }

    /// Create revoke token permissions request builder.
    /// This method is used to revoke token permissions.
    ///
    /// Instance of [`RevokeTokenRequestWithDeserializerBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    ///     core::{Deserializer, PubNubError, Serializer},
    /// #    PubNubClientBuilder, Keyset,
    /// };
    ///
    /// struct MyDeserializer;
    ///
    /// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
    ///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
    ///         // ...
    /// #        Ok(GrantTokenResult)
    ///     }
    /// }
    ///
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .revoke_token("p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI".into())
    ///     .derialize_with(MyDeserializer)
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "serde"))]
    pub fn revoke_token<S>(&self, token: S) -> RevokeTokenRequestWithDeserializerBuilder<T>
    where
        S: Into<String>,
    {
        RevokeTokenRequestWithDeserializerBuilder {
            pubnub_client: self.clone(),
            token: token.into(),
        }
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{
        core::{PubNubError, TransportMethod, TransportRequest, TransportResponse},
        transport::middleware::PubNubMiddleware,
        Keyset,
    };
    use std::collections::HashMap;

    /// Requests handler function type.
    type RequestHandler = Box<dyn Fn(&TransportRequest) + Send + Sync>;

    #[derive(Default)]
    struct MockTransport {
        ///  Response which mocked transport should return.
        response: Option<TransportResponse>,

        /// Request handler function which will be called before returning
        /// response.
        ///
        /// Use function to verify request parameters.
        request_handler: Option<RequestHandler>,
    }

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            // Calling request handler (if provided).
            if let Some(handler) = &self.request_handler {
                handler(&req);
            }

            Ok(self
                .response
                .clone()
                .unwrap_or(transport_response(200, None)))
        }
    }

    /// Service response payload.
    fn transport_response(status: u16, token: Option<String>) -> TransportResponse {
        let error = "\"error\":{{\"message\":\"Overall error\",\"source\":\"test\",\
        \"details\":[{{\"message\":\"Error\",\"location\":\"signature\",\"locationType\":\"query\"}}]}}";
        let data = format!(
            "\"data\":{{\"message\":\"Success\"{}}}}}",
            token.map_or(String::new(), |t| format!(",\"token\":\"{}\"", t))
        );

        TransportResponse {
            status,
            body: Some(Vec::from(format!(
                "{{\"status\":{},\"service\":\"Access Manager\",{}",
                status,
                if status < 400 { data } else { error.to_owned() }
            ))),
            ..Default::default()
        }
    }

    /// List of default permissions.
    fn permissions() -> Vec<Box<dyn permissions::Permission>> {
        vec![
            permissions::channel("channel").read().update(),
            permissions::user_id("id").get().delete(),
        ]
    }

    /// Construct test client with mocked transport.
    fn client(
        with_subscribe_key: bool,
        with_secret_key: bool,
        transport: Option<MockTransport>,
    ) -> PubNubClient<PubNubMiddleware<MockTransport>> {
        PubNubClient::with_transport(transport.unwrap_or(MockTransport {
            response: None,
            request_handler: None,
        }))
        .with_keyset(Keyset {
            subscribe_key: if with_subscribe_key { "demo" } else { "" },
            publish_key: Some(""),
            secret_key: with_secret_key.then_some("demo"),
        })
        .with_user_id("user")
        .build()
        .unwrap()
    }

    #[test]
    fn not_grant_token_when_subscribe_key_missing() {
        let permissions = permissions();
        let client = client(false, true, None);
        let request = client.grant_token(10).resources(&permissions).build();

        assert!(&client.config.subscribe_key.is_empty());
        assert!(request.is_err());
    }

    #[test]
    fn not_grant_token_when_secret_key_missing() {
        let permissions = permissions();
        let client = client(true, false, None);
        let request = client.grant_token(10).resources(&permissions).build();

        assert!(client
            .config
            .secret_key
            .as_deref()
            .unwrap_or_default()
            .is_empty());
        assert!(request.is_err());
    }

    #[tokio::test]
    async fn grant_token() {
        let permissions = permissions();
        let transport = MockTransport {
            response: Some(transport_response(200, Some("test-token".to_string()))),
            ..Default::default()
        };
        let client = client(true, true, Some(transport));
        let result = client
            .grant_token(10)
            .resources(&permissions)
            .execute()
            .await;

        match result {
            Ok(response) => assert_eq!(response.token, "test-token"),
            Err(err) => panic!("Request should not fail: {}", err),
        }
    }

    #[tokio::test]
    async fn include_timestamp_in_query_for_grant_token() {
        let permissions = permissions();
        let transport = MockTransport {
            response: None,
            request_handler: Some(Box::new(|req| {
                assert!(req.query_parameters.contains_key("timestamp"));
                assert!(req.query_parameters.get("timestamp").is_some());
            })),
        };

        let _ = client(true, true, Some(transport))
            .grant_token(10)
            .resources(&permissions)
            .execute()
            .await;
    }

    #[tokio::test]
    async fn include_signature_in_query_for_grant_token() {
        let permissions = permissions();
        let transport = MockTransport {
            response: None,
            request_handler: Some(Box::new(|req| {
                assert!(req.query_parameters.contains_key("signature"));
                assert!(req.query_parameters.get("signature").is_some());
                assert!(req
                    .query_parameters
                    .get("signature")
                    .unwrap()
                    .contains("v2."));
            })),
        };

        let _ = client(true, true, Some(transport))
            .grant_token(10)
            .resources(&permissions)
            .execute()
            .await;
    }

    #[test]
    fn include_body_for_grant_token() {
        let permissions = permissions();
        let request = client(true, true, None)
            .grant_token(10)
            .resources(&permissions)
            .meta(HashMap::from([
                ("string".into(), "string-value".into()),
                ("integer".into(), 465.into()),
                ("float".into(), 15.89.into()),
                ("boolean".into(), true.into()),
                ("null".into(), ().into()),
            ]))
            .build()
            .unwrap()
            .transport_request();

        // Serialization order is not constant. so ensure thar required
        // key/value pairs is present in body.
        let body = String::from_utf8(request.body.unwrap()).unwrap_or("".into());
        assert!(body.contains("\"string\":\"string-value\""));
        assert!(body.contains("\"boolean\":true"));
        assert!(body.contains("\"null\":null"));
        assert!(body.contains("\"integer\":465"));
        assert!(body.contains("\"float\":15.89"));
        assert!(body.contains("\"channels\":{\"channel\":65}"));
        assert!(body.contains("\"uuids\":{\"id\":40}"));
        assert!(matches!(&request.method, TransportMethod::Post));
    }

    #[test]
    fn not_revoke_token_when_subscribe_key_missing() {
        let client = client(false, true, None);
        let request = client.revoke_token("test/to+en==").build();

        assert!(&client.config.subscribe_key.is_empty());
        assert!(request.is_err());
    }

    #[test]
    fn not_revoke_token_when_secret_key_missing() {
        let client = client(true, false, None);
        let request = client.revoke_token("test/to+en==").build();

        assert!(client
            .config
            .secret_key
            .as_deref()
            .unwrap_or_default()
            .is_empty());
        assert!(request.is_err());
    }

    #[tokio::test]
    async fn revoke_token() {
        let client = client(true, true, None);
        let result = client.revoke_token("test/to+en==").execute().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn include_encoded_token_in_path_for_revoke_token() {
        let request = client(true, true, None)
            .revoke_token("test/to+en==")
            .build()
            .unwrap()
            .transport_request();
        assert!(request.path.ends_with("test%2Fto%2Ben%3D%3D"));
        assert!(matches!(&request.method, TransportMethod::Delete));
    }
}
