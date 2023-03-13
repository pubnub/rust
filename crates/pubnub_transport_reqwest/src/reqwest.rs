use pubnub_core::{Transport, TransportMethod, TransportRequest};
use pubnub_core::transport_response::TransportResponse;

struct TransportReqwest {
    reqwest_client: reqwest::Client,
    hostname: String,
}

#[async_trait::async_trait]
impl Transport for TransportReqwest {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, ()> {

        let path = req.query_parameters.iter().fold(format!("{}?", req.path), |url, (k, v)| {
            format!("{}{}={}&", url, k, v)
        });
        match req.method {
            TransportMethod::Get => {
                self.reqwest_client.get(format!("{}{}", &self.hostname, path))
                    .send()
                    .await
                    .map(|reqwest_response| {
                        TransportResponse {
                            status: reqwest_response.status().as_u16(),
                            body: None,//Some(reqwest_response.bytes().await.as_ref().unwrap().to_vec()),
                            ..Default::default()
                        }
                    })
                    .map_err(|_e| ())
            }
            TransportMethod::Post => {
                todo!()
            }
        }

    }
}


#[cfg(test)]
mod should {
    use pubnub_core::TransportMethod::Get;
    use pubnub_core::{Transport, TransportRequest};
    use crate::reqwest::TransportReqwest;

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
}