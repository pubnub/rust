//! # Reqwest Transport Implementation
//!
//! This module contains the [`TransportReqwest`] struct.
//! It is used to send requests to the [`PubNub API`] using the [`reqwest`] crate.
//! It is intended to be used by the [`pubnub`] crate.
//!
//! It requires the [`reqwest` feature] to be enabled.
//!
//! [`TransportReqwest`]: ./struct.TransportReqwest.html
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`reqwest`]: https://docs.rs/reqwest
//! [`pubnub`]: ../index.html
//! [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
//! [`reqwest` feature]: ../index.html#features

use crate::{
    core::{
        error::{PubNubError, PubNubError::TransportError},
        Transport, TransportMethod, TransportRequest, TransportResponse,
    },
    PubNubClientBuilder,
};
use log::info;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use urlencoding::encode;

/// This struct is used to send requests to the [`PubNub API`] using the [`reqwest`] crate.
/// It is used as the transport type for the [`PubNubClient`].
/// It is intended to be used by the [`pubnub`] crate.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
/// [`reqwest`]: https://docs.rs/reqwest
/// [`pubnub`]: ../index.html
/// [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
#[derive(Clone, Debug)]
pub struct TransportReqwest {
    reqwest_client: reqwest::Client,

    /// The hostname to use for requests.
    /// It is used as the base URL for all requests.
    ///
    /// It defaults to `https://ps.pndsn.com/`.
    /// # Examples
    /// ```
    /// use pubnub::transport::TransportReqwest;
    ///
    /// let transport = {
    ///    let mut transport = TransportReqwest::default();
    ///    transport.hostname = "https://wherever.you.want.com/".into();
    ///    transport
    /// };
    /// ```
    pub hostname: String,
}

#[async_trait::async_trait]
impl Transport for TransportReqwest {
    async fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let request_url = prepare_url(&self.hostname, &request.path, &request.query_parameters);
        info!("{}", request_url);
        let headers = prepare_headers(&request.headers)?;
        let builder = match request.method {
            TransportMethod::Get => self.prepare_get_method(request, request_url),
            TransportMethod::Post => self.prepare_post_method(request, request_url),
        }?;

        let result = builder
            .headers(headers)
            .send()
            .await
            .map_err(|e| TransportError(e.to_string()))?;

        Ok(TransportResponse {
            status: result.status().as_u16(),
            body: if result.content_length().is_some() {
                Some(
                    result
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| TransportError(e.to_string()))?,
                )
            } else {
                None
            },
            ..Default::default()
        })
    }
}

impl Default for TransportReqwest {
    fn default() -> Self {
        Self {
            reqwest_client: reqwest::Client::default(),
            hostname: "https://ps.pndsn.com/".into(),
        }
    }
}

impl TransportReqwest {
    /// Create a new [`TransportReqwest`] instance.
    /// It is used as the transport type for the [`PubNubClient`].
    /// It is intended to be used by the [`pubnub`] crate.
    /// It is used by the [`PubNubClientBuilder`] to create a [`PubNubClient`].
    ///
    /// It provides a default [`reqwest`] client using [`reqwest::Client::default()`]
    /// and a default hostname of `https://ps.pndsn.com`.
    ///
    /// # Example
    /// ```
    /// use pubnub::transport::TransportReqwest;
    ///
    /// let transport = TransportReqwest::new();
    /// ```
    ///
    /// [`TransportReqwest`]: ./struct.TransportReqwest.html
    /// [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
    /// [`pubnub`]: ../index.html
    /// [`PubNubClientBuilder`]: ../pubnub_client/struct.PubNubClientBuilder.html
    /// [`reqwest`]: https://docs.rs/reqwest
    pub fn new() -> Self {
        Self::default()
    }

    fn prepare_get_method(
        &self,
        _request: TransportRequest,
        url: String,
    ) -> Result<reqwest::RequestBuilder, PubNubError> {
        Ok(self.reqwest_client.get(url))
    }

    fn prepare_post_method(
        &self,
        request: TransportRequest,
        url: String,
    ) -> Result<reqwest::RequestBuilder, PubNubError> {
        request
            .body
            .ok_or(TransportError("Body should not be empty for POST".into()))
            .map(|vec_bytes| self.reqwest_client.post(url).body(vec_bytes))
    }
}

fn prepare_headers(request_headers: &HashMap<String, String>) -> Result<HeaderMap, PubNubError> {
    HeaderMap::try_from(request_headers).map_err(|err| PubNubError::TransportError(err.to_string()))
}

// TODO: create test for merging query params
fn prepare_url(hostname: &str, path: &str, query_params: &HashMap<String, String>) -> String {
    if query_params.is_empty() {
        return format!("{}{}", hostname, path);
    }
    let mut qp = query_params
        .iter()
        .fold(format!("{}{}?", hostname, path), |acc_query, (k, v)| {
            format!("{}{}={}&", acc_query, k, encode(v))
        });

    qp.remove(qp.len() - 1);
    qp
}

impl PubNubClientBuilder<TransportReqwest> {
    /// Creates a new [`PubNubClientBuilder`] with the default [`TransportReqwest`] transport.
    /// The default transport uses the [`reqwest`] crate to send requests to the [`PubNub API`].
    /// The default hostname is `https://ps.pndsn.com`.
    /// The default [`reqwest`] client is created using [`reqwest::Client::default()`].
    ///
    /// # Examples
    /// ```
    /// use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// let client = PubNubClientBuilder::with_reqwest_transport()
    ///     .with_keyset(Keyset {
    ///         subscribe_key: "sub-c-abc123",
    ///         publish_key: Some("pub-c-abc123"),
    ///         secret_key: None,
    ///     })
    ///     .with_user_id("user-123")
    ///     .build();
    /// ```
    ///
    /// [`PubNubClientBuilder`]: ../pubnub_client/struct.PubNubClientBuilder.html
    /// [`TransportReqwest`]: ./struct.TransportReqwest.html
    /// [`reqwest`]: https://docs.rs/reqwest
    /// [`PubNub API`]: https://www.pubnub.com/docs
    /// [`PubNubClient`]: ../pubnub_client/struct.PubNubClient.html
    pub fn with_reqwest_transport() -> PubNubClientBuilder<TransportReqwest> {
        PubNubClientBuilder {
            transport: Some(TransportReqwest::new()),
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use test_case::test_case;
    use wiremock::matchers::{body_string, header, method, path as path_macher};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test_case("/path/%22Hello%22", "/path/\"Hello\"" ; "sending string")]
    #[test_case("/path/%7B%22a%22:%22b%22%7D", "/path/{\"a\":\"b\"}" ; "sending object")]
    #[test_case("/path/1", "/path/1" ; "sending number")]
    #[test_case("/path/true", "/path/true" ; "sending boolean")]
    #[test_case("/path/[%22a%22]", "/path/[\"a\"]" ; "sending array")]
    #[tokio::test]
    async fn send_via_get_method(path_to_match: &str, path_to_send: &str) {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_macher(path_to_match))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("[1,\"Sent\",\"16787176144828000\"]"),
            )
            .mount(&server)
            .await;

        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: server.uri(),
        };

        let request = TransportRequest {
            path: path_to_send.into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: TransportMethod::Get,
            body: None,
            ..Default::default()
        };

        let response = transport.send(request).await.unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn send_via_post_method() {
        let message = "\"Hello from post\"";
        let path = "/publish/sub_key/pub_key/0/chat/0";

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_macher(path))
            .and(body_string(message.to_string()))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("[1,\"Sent\",\"16787176144828000\"]"),
            )
            .mount(&server)
            .await;

        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: server.uri(),
        };

        let request = TransportRequest {
            path: path.into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: TransportMethod::Post,
            body: Some(message.chars().map(|c| c as u8).collect()),
            ..Default::default()
        };

        let response = transport.send(request).await.unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn send_headers() {
        let path = "/publish/sub_key/pub_key/0/chat/0";
        let expected_key = "k";
        let expected_val = "v";

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_macher(path))
            .and(header(expected_key, expected_val))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("[1,\"Sent\",\"16787176144828000\"]"),
            )
            .mount(&server)
            .await;

        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: server.uri(),
        };

        let request = TransportRequest {
            path: path.into(),
            method: TransportMethod::Get,
            headers: HashMap::from([(expected_key.into(), expected_val.into())]),
            ..Default::default()
        };

        let response = transport.send(request).await.unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn return_err_on_post_empty_body() {
        let transport = TransportReqwest::default();

        let request = TransportRequest {
            method: TransportMethod::Post,
            body: None,
            ..Default::default()
        };

        assert!(transport.send(request).await.is_err());
    }
}
