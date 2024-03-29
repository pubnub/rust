//! # Reqwest Transport Implementation
//!
//! This module contains the [`TransportReqwest`] struct.
//! It is used to send requests to the [`PubNub API`] using the [`reqwest`]
//! crate. It is intended to be used by the [`pubnub`] crate.
//!
//! It requires the [`reqwest` feature] to be enabled.
//!
//! [`TransportReqwest`]: ./struct.TransportReqwest.html
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`reqwest`]: https://docs.rs/reqwest
//! [`pubnub`]: ../index.html
//! [`reqwest` feature]: ../index.html#features

#[cfg(any(
    all(not(feature = "subscribe"), not(feature = "presence")),
    not(feature = "std")
))]
use crate::dx::pubnub_client::PubNubClientDeserializerBuilder;

#[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
use crate::dx::pubnub_client::PubNubClientRuntimeBuilder;

use crate::{
    core::{
        error::PubNubError, transport::PUBNUB_DEFAULT_BASE_URL, utils::encoding::url_encode,
        Transport, TransportMethod, TransportRequest, TransportResponse,
    },
    lib::{
        alloc::{
            boxed::Box,
            format,
            string::{String, ToString},
        },
        collections::HashMap,
    },
    PubNubClientBuilder,
};
use bytes::Bytes;
use log::info;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue},
    StatusCode,
};

/// This struct is used to send requests to the [`PubNub API`] using the
/// [`reqwest`] crate. It is used as the transport type for the
/// [`PubNubClient`]. It is intended to be used by the [`pubnub`] crate.
///
/// [`PubNubClient`]: ../../dx/pubnub_client/struct.PubNubClientInstance.html
/// [`PubNub API`]: https://www.pubnub.com/docs
/// [`reqwest`]: https://docs.rs/reqwest
/// [`pubnub`]: ../index.html
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
    ///    transport.hostname = "https://wherever.you.want.com".into();
    ///    transport
    /// };
    /// ```
    pub hostname: String,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Transport for TransportReqwest {
    async fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let request_url = prepare_url(&self.hostname, &request.path, &request.query_parameters);
        info!(
            "Sending data to pubnub: {} {:?} {}",
            request.method, request.headers, request_url
        );

        let headers = prepare_headers(&request.headers)?;
        #[cfg(feature = "std")]
        let timeout = request.timeout;

        #[cfg(feature = "std")]
        let mut builder = match request.method {
            TransportMethod::Get => self.prepare_get_method(request, request_url),
            TransportMethod::Post => self.prepare_post_method(request, request_url),
            TransportMethod::Delete => self.prepare_delete_method(request, request_url),
        }?;

        #[cfg(feature = "std")]
        if timeout.gt(&0) {
            builder = builder.timeout(core::time::Duration::from_secs(timeout));
        }

        #[cfg(not(feature = "std"))]
        let builder = match request.method {
            TransportMethod::Get => self.prepare_get_method(request, request_url),
            TransportMethod::Post => self.prepare_post_method(request, request_url),
            TransportMethod::Delete => self.prepare_delete_method(request, request_url),
        }?;

        let result = builder
            .headers(headers)
            .send()
            .await
            .map_err(|e| PubNubError::Transport {
                details: e.to_string(),
                response: None,
            })?;

        let headers = result.headers().clone();
        let status = result.status();
        result
            .bytes()
            .await
            .map_err(|e| PubNubError::Transport {
                details: e.to_string(),
                response: Some(Box::new(TransportResponse {
                    status: status.into(),
                    headers: extract_headers(&headers),
                    body: None,
                })),
            })
            .and_then(|bytes| create_result(status, bytes, &headers))
    }
}

impl Default for TransportReqwest {
    fn default() -> Self {
        Self {
            reqwest_client: reqwest::Client::default(),
            hostname: PUBNUB_DEFAULT_BASE_URL.into(),
        }
    }
}

impl TransportReqwest {
    /// Create a new [`TransportReqwest`] instance.
    /// It is used as the transport type for the [`PubNubClient`].
    /// It is intended to be used by the [`pubnub`] crate.
    /// It is used by the [`PubNubClientBuilder`] to create a [`PubNubClient`].
    ///
    /// It provides a default [`reqwest`] client using
    /// [`reqwest::Client::default()`] and a default hostname of `https://ps.pndsn.com`.
    ///
    /// # Example
    /// ```
    /// use pubnub::transport::TransportReqwest;
    ///
    /// let transport = TransportReqwest::new();
    /// ```
    ///
    /// [`PubNubClient`]: ../../dx/pubnub_client/struct.PubNubClientInstance.html
    /// [`TransportReqwest`]: ./struct.TransportReqwest.html
    /// [`pubnub`]: ../index.html
    /// [`reqwest`]: https://docs.rs/reqwest
    pub fn new() -> Self {
        Self::default()
    }

    /// set the custom hostname for request
    pub fn set_hostname<S>(&mut self, hostname: S)
    where
        S: Into<String>,
    {
        self.hostname = hostname.into();
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
            .ok_or(PubNubError::Transport {
                details: "Body should not be empty for POST".into(),
                response: None,
            })
            .map(|vec_bytes| self.reqwest_client.post(url).body(vec_bytes))
    }

    fn prepare_delete_method(
        &self,
        _request: TransportRequest,
        url: String,
    ) -> Result<reqwest::RequestBuilder, PubNubError> {
        Ok(self.reqwest_client.delete(url))
    }
}

fn prepare_headers(request_headers: &HashMap<String, String>) -> Result<HeaderMap, PubNubError> {
    request_headers
        .iter()
        .map(|(k, v)| -> Result<(HeaderName, HeaderValue), PubNubError> {
            let name =
                TryFrom::try_from(k).map_err(|err: InvalidHeaderName| PubNubError::Transport {
                    details: err.to_string(),
                    response: None,
                })?;
            let value: HeaderValue =
                TryFrom::try_from(v).map_err(|err: InvalidHeaderValue| PubNubError::Transport {
                    details: err.to_string(),
                    response: None,
                })?;
            Ok((name, value))
        })
        .collect()
}

fn prepare_url(hostname: &str, path: &str, query_params: &HashMap<String, String>) -> String {
    if query_params.is_empty() {
        return format!("{}{}", hostname, path);
    }
    let mut qp = query_params
        .iter()
        .fold(format!("{}{}?", hostname, path), |acc_query, (k, v)| {
            format!("{}{}={}&", acc_query, k, url_encode(v.as_bytes()))
        });

    qp.remove(qp.len() - 1);
    qp
}

fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .fold(HashMap::new(), |mut acc, (name, value)| {
            if let Ok(value) = value.to_str() {
                acc.insert(name.to_string(), value.to_string());
            }
            acc
        })
}

fn create_result(
    status: StatusCode,
    body: Bytes,
    headers: &HeaderMap,
) -> Result<TransportResponse, PubNubError> {
    Ok(TransportResponse {
        status: status.as_u16(),
        body: (!body.is_empty()).then(|| body.to_vec()),
        headers: extract_headers(headers),
    })
}

impl PubNubClientBuilder {
    /// Creates a new [`PubNubClientBuilder`] with the default
    /// [`TransportReqwest`] transport. The default transport uses the
    /// [`reqwest`] crate to send requests to the [`PubNub API`]. The default hostname is `https://ps.pndsn.com`.
    /// The default [`reqwest`] client is created using
    /// [`reqwest::Client::default()`].
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
    /// [`TransportReqwest`]: ./struct.TransportReqwest.html
    /// [`reqwest`]: https://docs.rs/reqwest
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    pub fn with_reqwest_transport() -> PubNubClientRuntimeBuilder<TransportReqwest> {
        PubNubClientRuntimeBuilder {
            transport: TransportReqwest::new(),
        }
    }

    /// Creates a new [`PubNubClientBuilder`] with the default
    /// [`TransportReqwest`] transport. The default transport uses the
    /// [`reqwest`] crate to send requests to the [`PubNub API`]. The default hostname is `https://ps.pndsn.com`.
    /// The default [`reqwest`] client is created using
    /// [`reqwest::Client::default()`].
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
    /// [`TransportReqwest`]: ./struct.TransportReqwest.html
    /// [`reqwest`]: https://docs.rs/reqwest
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(any(
        all(not(feature = "subscribe"), not(feature = "presence")),
        not(feature = "std")
    ))]
    pub fn with_reqwest_transport() -> PubNubClientDeserializerBuilder<TransportReqwest> {
        PubNubClientDeserializerBuilder {
            transport: TransportReqwest::new(),
        }
    }
}

// blocking calls are disabled for reqwest on WASM target
#[cfg(all(feature = "blocking", not(target_arch = "wasm32")))]
pub mod blocking {
    //! # Reqwest Transport Blocking Implementation
    //!
    //! This module contains the [`TransportReqwest`] struct.
    //! It is used to send requests to the [`PubNub API`] using the [`reqwest`]
    //! crate. It is intended to be used by the [`pubnub`] crate.
    //!
    //! It requires the [`reqwest` and `blocking` feature] to be enabled.
    //!
    //! [`TransportReqwest`]: ./struct.TransportReqwest.html
    //! [`PubNub API`]: https://www.pubnub.com/docs
    //! [`reqwest`]: https://docs.rs/reqwest
    //! [`pubnub`]: ../index.html
    //! [`reqwest` feature]: ../index.html#features

    #[cfg(any(
        all(not(feature = "subscribe"), not(feature = "presence")),
        not(feature = "std")
    ))]
    use crate::dx::pubnub_client::PubNubClientDeserializerBuilder;

    #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
    use crate::dx::pubnub_client::PubNubClientRuntimeBuilder;

    use crate::{
        core::{
            transport::PUBNUB_DEFAULT_BASE_URL, PubNubError, TransportMethod, TransportRequest,
            TransportResponse,
        },
        lib::alloc::{
            boxed::Box,
            string::{String, ToString},
        },
        transport::reqwest::{create_result, extract_headers, prepare_headers, prepare_url},
        PubNubClientBuilder,
    };
    use log::info;

    /// This struct is used to send requests to the [`PubNub API`] using the
    /// [`reqwest`] crate. It is used as the transport type for the
    /// [`PubNubClient`]. It is intended to be used by the [`pubnub`] crate.
    ///
    /// It requires the [`reqwest` and `blocking` feature] to be enabled.
    ///
    /// [`PubNubClient`]: ../../dx/pubnub_client/struct.PubNubClientInstance.html
    /// [`PubNub API`]: https://www.pubnub.com/docs
    /// [`reqwest`]: https://docs.rs/reqwest
    /// [`pubnub`]: ../index.html
    pub struct TransportReqwest {
        reqwest_client: reqwest::blocking::Client,
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
        ///    transport.hostname = "https://wherever.you.want.com".into();
        ///    transport
        /// };
        /// ```
        pub hostname: String,
    }

    impl crate::core::blocking::Transport for TransportReqwest {
        fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
            let request_url = prepare_url(&self.hostname, &request.path, &request.query_parameters);
            info!(
                "Sending data to pubnub: {} {:?} {}",
                request.method, request.headers, request_url
            );
            let headers = prepare_headers(&request.headers)?;
            #[cfg(feature = "std")]
            let timeout = request.timeout;

            #[cfg(feature = "std")]
            let mut builder = match request.method {
                TransportMethod::Get => self.prepare_get_method(request, request_url),
                TransportMethod::Post => self.prepare_post_method(request, request_url),
                TransportMethod::Delete => self.prepare_delete_method(request, request_url),
            }?;

            #[cfg(feature = "std")]
            if timeout.gt(&0) {
                builder = builder.timeout(core::time::Duration::from_micros(timeout))
            }

            #[cfg(not(feature = "std"))]
            let builder = match request.method {
                TransportMethod::Get => self.prepare_get_method(request, request_url),
                TransportMethod::Post => self.prepare_post_method(request, request_url),
                TransportMethod::Delete => self.prepare_delete_method(request, request_url),
            }?;

            let result = builder
                .headers(headers)
                .send()
                .map_err(|e| PubNubError::Transport {
                    details: e.to_string(),
                    response: None,
                })?;

            let headers = result.headers().clone();
            let status = result.status();
            result
                .bytes()
                .map_err(|e| PubNubError::Transport {
                    details: e.to_string(),
                    response: Some(Box::new(TransportResponse {
                        status: status.into(),
                        headers: extract_headers(&headers),
                        body: None,
                    })),
                })
                .and_then(|bytes| create_result(status, bytes, &headers))
        }
    }

    impl Default for TransportReqwest {
        fn default() -> Self {
            Self {
                reqwest_client: reqwest::blocking::Client::default(),
                hostname: PUBNUB_DEFAULT_BASE_URL.into(),
            }
        }
    }

    impl TransportReqwest {
        /// Create a new [`TransportReqwest`] instance.
        /// It is used as the transport type for the [`PubNubClient`].
        /// It is intended to be used by the [`pubnub`] crate.
        /// It is used by the [`PubNubClientBuilder`] to create a
        /// [`PubNubClient`].
        ///
        /// It provides a default [`reqwest`] client using
        /// [`reqwest::Client::default()`] and a default hostname of `https://ps.pndsn.com`.
        ///
        /// # Example
        /// ```
        /// use pubnub::transport::TransportReqwest;
        ///
        /// let transport = TransportReqwest::new();
        /// ```
        ///
        /// [`PubNubClient`]: ../../dx/pubnub_client/struct.PubNubClientInstance.html
        /// [`TransportReqwest`]: ./struct.TransportReqwest.html
        /// [`pubnub`]: ../index.html
        /// [`reqwest`]: https://docs.rs/reqwest
        pub fn new() -> Self {
            Self::default()
        }

        fn prepare_get_method(
            &self,
            _request: TransportRequest,
            request_url: String,
        ) -> Result<reqwest::blocking::RequestBuilder, PubNubError> {
            let builder = self.reqwest_client.get(request_url);
            Ok(builder)
        }

        fn prepare_post_method(
            &self,
            request: TransportRequest,
            request_url: String,
        ) -> Result<reqwest::blocking::RequestBuilder, PubNubError> {
            let builder = self.reqwest_client.post(request_url);
            let builder = match request.body {
                Some(body) => builder.body(body),
                None => builder,
            };
            Ok(builder)
        }

        fn prepare_delete_method(
            &self,
            _request: TransportRequest,
            request_url: String,
        ) -> Result<reqwest::blocking::RequestBuilder, PubNubError> {
            Ok(self.reqwest_client.delete(request_url))
        }
    }

    impl PubNubClientBuilder {
        /// Creates a new [`PubNubClientBuilder`] with the default
        /// [`TransportReqwest`] transport. The default transport uses
        /// the [`reqwest`] crate to send requests to the [`PubNub API`]. The default hostname is `https://ps.pndsn.com`.
        /// The default [`reqwest`] client is created using
        /// [`reqwest::Client::default()`].
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
        /// [`TransportReqwest`]: ./struct.TransportReqwest.html
        /// [`reqwest`]: https://docs.rs/reqwest
        /// [`PubNub API`]: https://www.pubnub.com/docs
        #[cfg(all(any(feature = "subscribe", feature = "presence"), feature = "std"))]
        pub fn with_reqwest_blocking_transport() -> PubNubClientRuntimeBuilder<TransportReqwest> {
            PubNubClientRuntimeBuilder {
                transport: TransportReqwest::new(),
            }
        }

        /// Creates a new [`PubNubClientBuilder`] with the default
        /// [`TransportReqwest`] transport. The default transport uses
        /// the [`reqwest`] crate to send requests to the [`PubNub API`]. The default hostname is `https://ps.pndsn.com`.
        /// The default [`reqwest`] client is created using
        /// [`reqwest::Client::default()`].
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
        /// [`TransportReqwest`]: ./struct.TransportReqwest.html
        /// [`reqwest`]: https://docs.rs/reqwest
        /// [`PubNub API`]: https://www.pubnub.com/docs
        #[cfg(any(
            all(not(feature = "subscribe"), not(feature = "presence")),
            not(feature = "std")
        ))]
        pub fn with_reqwest_blocking_transport() -> PubNubClientDeserializerBuilder<TransportReqwest>
        {
            PubNubClientDeserializerBuilder {
                transport: TransportReqwest::new(),
            }
        }
    }

    #[cfg(test)]
    mod should {
        use super::*;
        use crate::{core::blocking::Transport, lib::alloc::string::ToString};

        use test_case::test_case;
        use wiremock::matchers::{body_string, method, path as path_macher};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test_case("/path/%22Hello%22", "/path/\"Hello\"" ; "sending string")]
        #[test_case("/path/%7B%22a%22:%22b%22%7D", "/path/{\"a\":\"b\"}" ; "sending object")]
        #[test_case("/path/1", "/path/1" ; "sending number")]
        #[test_case("/path/true", "/path/true" ; "sending boolean")]
        #[test_case("/path/[%22a%22]", "/path/[\"a\"]" ; "sending array")]
        #[tokio::test]
        async fn send_via_get_method(path_to_match: &str, path_to_send: &str) {
            let server = MockServer::start().await;
            let path_to_send = path_to_send.to_string();

            Mock::given(method("GET"))
                .and(path_macher(path_to_match))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_string("[1,\"Sent\",\"16787176144828000\"]"),
                )
                .mount(&server)
                .await;

            tokio::task::spawn_blocking(move || {
                let transport = TransportReqwest {
                    reqwest_client: reqwest::blocking::Client::default(),
                    hostname: server.uri(),
                };

                let request = TransportRequest {
                    path: path_to_send,
                    query_parameters: [("uuid".into(), "Phoenix".into())].into(),
                    method: TransportMethod::Get,
                    body: None,
                    ..Default::default()
                };

                let response = transport.send(request).unwrap();

                assert_eq!(response.status, 200);
            })
            .await
            .unwrap();
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
                    ResponseTemplate::new(200)
                        .set_body_string("[1,\"Sent\",\"16787176144828000\"]"),
                )
                .mount(&server)
                .await;

            tokio::task::spawn_blocking(move || {
                let transport = TransportReqwest {
                    reqwest_client: reqwest::blocking::Client::default(),
                    hostname: server.uri(),
                };

                let request = TransportRequest {
                    path: path.into(),
                    query_parameters: [("uuid".into(), "Phoenix".into())].into(),
                    method: TransportMethod::Post,
                    body: Some(message.chars().map(|c| c as u8).collect()),
                    ..Default::default()
                };

                let response = transport.send(request).unwrap();

                assert_eq!(response.status, 200);
            })
            .await
            .unwrap();
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::lib::alloc::string::ToString;

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
            .and(path_macher(path_to_match.to_string()))
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

    #[test]
    fn verify_query_params_merge() {
        let query_params = HashMap::<String, String>::from([
            ("norep".into(), "true".into()),
            ("space-id".to_string(), "space_id".to_string()),
            ("meta".to_string(), "{\"k\":\"v\"}".to_string()),
            ("seqn".to_string(), "1".to_string()),
            ("pnsdk".to_string(), "rust/1.2".to_string()),
        ]);

        let url_string = prepare_url("host:8080", "/key/channel", &query_params);
        let parsed_url = reqwest::Url::parse(&url_string).unwrap();
        let retrived_query_params: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        assert_eq!(query_params, retrived_query_params);
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
