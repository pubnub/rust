use pubnub_core::error::PubNubError;
use pubnub_core::error::PubNubError::TransportError;
use pubnub_core::transport_response::TransportResponse;
use pubnub_core::{Transport, TransportMethod, TransportRequest};
use reqwest::Response;
use std::collections::HashMap;

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
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let path = prepare_path(req.path, req.query_parameters);

        let result = match req.method {
            TransportMethod::Get => self
                .reqwest_client
                .get(format!("{}{}", &self.hostname, path))
                .send()
                .await
                .map_err(|e| TransportError(e.to_string())),
            TransportMethod::Post => match req.body {
                None => Err(TransportError(String::from(
                    "Body should not be empty for POST",
                ))),
                Some(vec_bytes) => self
                    .reqwest_client
                    .post(format!("{}{}", &self.hostname, path))
                    .body(vec_bytes)
                    .send()
                    .await
                    .map_err(|e| TransportError(e.to_string())),
            },
        };

        let reqwest_response = result?;

        Ok(TransportResponse {
            status: reqwest_response.status().as_u16(),
            body: if reqwest_response.content_length().is_some() {
                Some(
                    reqwest_response
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

#[cfg(test)]
mod should {
    use crate::reqwest::TransportReqwest;
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use pubnub_core::TransportMethod::{Get, Post};
    use pubnub_core::{Transport, TransportRequest};

    #[tokio::test]
    async fn send_via_get_method() {
        let server = MockServer::start();
        let message = "\"Hello\"";
        let path = "/publish/sub_key/pub_key/0/chat/0/";
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
            path: format!("{}{}", path, message).into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: Get,
            body: None,
            headers: [].into(),
        };
        let response = transport.send(request).await.unwrap();
        hello_mock.assert();
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn send_via_post_method() {
        let server = MockServer::start();
        let message = "\"Hello from post\"";
        let path = "/publish/sub_key/pub_key/0/chat/0";
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
            body: Some(String::from(message).chars().map(|c| c as u8).collect()),
            headers: [].into(),
        };

        let response = transport.send(request).await.unwrap();
        hello_mock.assert();
        assert_eq!(response.status, 200);
    }
}
