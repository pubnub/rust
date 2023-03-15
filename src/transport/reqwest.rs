use crate::core::{
    error::{PubNubError, PubNubError::TransportError},
    Transport, TransportMethod, TransportRequest, TransportResponse,
};
use log::info;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct TransportReqwest {
    reqwest_client: reqwest::Client,
    hostname: String,
}

#[async_trait::async_trait]
impl Transport for TransportReqwest {
    async fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let request_url = prepare_url(&self.hostname, &request.path, &request.query_parameters);
        info!("{}", request_url);
        let result = match request.method {
            TransportMethod::Get => self.send_via_get_method(request, request_url).await,
            TransportMethod::Post => self.send_via_post_method(request, request_url).await,
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
        _request: TransportRequest,
        url: String,
    ) -> Result<reqwest::Response, PubNubError> {
        self.reqwest_client
            .get(url)
            .send()
            .await
            .map_err(|e| TransportError(e.to_string()))
    }

    async fn send_via_post_method(
        &self,
        request: TransportRequest,
        url: String,
    ) -> Result<reqwest::Response, PubNubError> {
        request
            .body
            .ok_or(TransportError("Body should not be empty for POST".into()))
            .map(|vec_bytes| self.reqwest_client.post(url).body(vec_bytes).send())?
            .await
            .map_err(|e| TransportError(e.to_string()))
    }
}

fn prepare_url(hostname: &str, path: &str, query_params: &HashMap<String, String>) -> String {
    if query_params.is_empty() {
        return format!("{}{}", hostname, path);
    }
    query_params
        .iter()
        .fold(format!("{}?", path), |acc_query, (k, v)| {
            format!("{}{}{}={}&", hostname, acc_query, k, v)
        })
}

#[cfg(test)]
mod should {
    use super::*;
    use wiremock::matchers::{body_string, method, path as path_macher};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_via_get_method() {
        let message = "\"Hello\"";
        let path = "/publish/sub_key/pub_key/0/chat/0/";

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_macher(format!(
                "{}{}",
                path,
                message.replace('\"', "%22")
            )))
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
            path: format!("{}{}", path, message),
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
    async fn return_err_on_post_empty_body() {
        let transport = TransportReqwest::default();

        let request = TransportRequest {
            body: None,
            ..Default::default()
        };

        assert!(transport.send(request).await.is_err());
    }
}
