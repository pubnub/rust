use pubnub_core::error::PubNubError;
use pubnub_core::error::PubNubError::TransportError;
use pubnub_core::transport_response::TransportResponse;
use pubnub_core::{Transport, TransportMethod, TransportRequest};

struct TransportReqwest {
    reqwest_client: reqwest::Client,
    hostname: String,
}

#[async_trait::async_trait]
impl Transport for TransportReqwest {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        let path = req
            .query_parameters
            .iter()
            .fold(format!("{}?", req.path), |url, (k, v)| {
                format!("{}{}={}&", url, k, v)
            });
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
                        .map_err(|e| PubNubError::TransportError(e.to_string()))?,
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
    use pubnub_core::TransportMethod::{Get, Post};
    use pubnub_core::{Transport, TransportRequest};

    #[tokio::test]
    async fn test_test_test() {
        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: "https://ps.pndsn.com".into(),
        };

        let request = TransportRequest {
            path: "/publish/demo-36/demo-36/0/chat/0/\"Hello\"".into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: Get,
            body: None,
            headers: [].into(),
        };

        println!("{:?}", transport.send(request).await.unwrap())
    }

    #[tokio::test]
    async fn test_via_post() {
        let transport = TransportReqwest {
            reqwest_client: reqwest::Client::default(),
            hostname: "https://ps.pndsn.com".into(),
        };

        let request = TransportRequest {
            path: "/publish/demo-36/demo-36/0/chat/0".into(),
            query_parameters: [("uuid".into(), "Phoenix".into())].into(),
            method: Post,
            body: Some(
                String::from("\"Hello from post\"")
                    .chars()
                    .map(|c| c as u8)
                    .collect(),
            ),
            headers: [].into(),
        };

        let result = transport.send(request).await.unwrap();

        println!("{:?}", String::from_utf8(result.body.unwrap()))
    }
}
