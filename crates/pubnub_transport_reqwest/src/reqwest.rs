use pubnub_core::error::PubNubError;
use pubnub_core::error::PubNubError::TransportError;
use pubnub_core::transport_response::TransportResponse;
use pubnub_core::{Transport, TransportMethod, TransportRequest};
use reqwest::Response;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct TransportReqwest {
    reqwest_client: reqwest::Client,
    hostname: String,
}

fn prepare_path(path: String, query_params: HashMap<String, String>) -> String {
    if query_params.is_empty() {
        return path;
    }
    query_params
        .iter()
        .fold(format!("{}?", path), |acc_query, (k, v)| {
            format!("{}{}={}&", acc_query, k, v)
        })
}

#[async_trait::async_trait]
impl Transport for TransportReqwest {
    async fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let result = match request.method {
            TransportMethod::Get => self.send_via_get_method(request).await,
            TransportMethod::Post => self.send_via_post_method(request).await,
        }?;

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

impl TransportReqwest {
    async fn send_via_get_method(
        &self,
        request: TransportRequest,
    ) -> Result<reqwest::Response, PubNubError> {
        self.reqwest_client
            .get(format!("{}{}", &self.hostname, request.path))
            .send()
            .await
            .map_err(|e| TransportError(e.to_string()))
    }

    async fn send_via_post_method(
        &self,
        request: TransportRequest,
    ) -> Result<reqwest::Response, PubNubError> {
        let path = prepare_path(request.path, request.query_parameters);

        request
            .body
            .ok_or(TransportError("Body should not be empty for POST".into()))
            .map(|vec_bytes| {
                self.reqwest_client
                    .post(format!("{}{}", &self.hostname, path))
                    .body(vec_bytes)
                    .send()
            })?
            .await
            .map_err(|e| TransportError(e.to_string()))
    }
}

#[cfg(test)]
mod should {
    use crate::reqwest::TransportReqwest;
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use pubnub_core::TransportMethod::{Get, Post};
    use pubnub_core::{Transport, TransportRequest};

    #[tokio::test]
    async fn send_via_get_method() {
        let message = "\"Hello\"";
        let path = "/publish/sub_key/pub_key/0/chat/0/";

        let server = MockServer::start();
        let hello_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{}{}", path, message.replace("\"", "%22")));
            then.status(200).body("[1,\"Sent\",\"16787176144828000\"]");
        });

        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: server.base_url(),
        };

        let request = TransportRequest {
            path: format!("{}{}", path, message),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: Get,
            body: None,
            ..Default::default()
        };

        let response = transport.send(request).await.unwrap();

        hello_mock.assert();
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn send_via_post_method() {
        let message = "\"Hello from post\"";
        let path = "/publish/sub_key/pub_key/0/chat/0";

        let server = MockServer::start();
        let hello_mock = server.mock(|when, then| {
            when.method(POST).path(path).body(message);
            then.status(200).body("[1,\"Sent\",\"16787176144828000\"]");
        });

        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: server.base_url(),
        };

        let request = TransportRequest {
            path: path.into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: Post,
            body: Some(message.chars().map(|c| c as u8).collect()),
            ..Default::default()
        };

        let response = transport.send(request).await.unwrap();

        hello_mock.assert();
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn return_err_on_post_empty_body() {
        let transport = TransportReqwest::default();

        let request = TransportRequest {
            body: None,
            ..Default::default()
        };

        assert!(transport.send(request).await.is_err());
    }
}
