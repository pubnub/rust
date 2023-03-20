//! TODO: docs

use crate::core::{PubNubError, Transport, TransportRequest, TransportResponse};
use crate::dx::pubnub_client::{SDK_ID, VERSION};
use uuid::Uuid;

/// TODO: Add docs
pub struct PubNubMiddleware<T>
where
    T: Transport,
{
    pub transport: T,
    pub include_request_id: bool,
    pub instance_id: Option<String>,
    pub user_id: String,
}

#[async_trait::async_trait]
impl<T> Transport for PubNubMiddleware<T>
where
    T: Transport + Sync + Send,
{
    async fn send(&self, mut req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        if self.include_request_id {
            req.query_parameters
                .insert("requestid".into(), Uuid::new_v4().to_string());
        }
        req.query_parameters
            .insert("pnsdk".into(), format!("{}/{}", SDK_ID, VERSION));
        req.query_parameters
            .insert("uuid".into(), self.user_id.clone());

        if let Some(instance_id) = &self.instance_id {
            req.query_parameters
                .insert("instanceid".into(), instance_id.clone());
        }

        self.transport.send(req).await
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::TransportResponse;

    #[tokio::test]
    async fn publish_message() {
        #[derive(Default)]
        struct MockTransport;

        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(
                &self,
                request: TransportRequest,
            ) -> Result<TransportResponse, PubNubError> {
                assert_eq!(
                    "user_id",
                    request.query_parameters.get("uuid").unwrap().clone()
                );
                assert_eq!(
                    "instance_id",
                    request.query_parameters.get("instanceid").unwrap().clone()
                );
                assert_eq!(
                    format!("{}/{}", SDK_ID, VERSION),
                    request.query_parameters.get("pnsdk").unwrap().clone()
                );
                assert!(request.query_parameters.contains_key("requestid"));
                Ok(TransportResponse::default())
            }
        }

        let middleware = PubNubMiddleware {
            transport: MockTransport::default(),
            include_request_id: true,
            instance_id: Some(String::from("instance_id")),
            user_id: "user_id".to_string(),
        };

        let result = middleware.send(TransportRequest::default()).await;

        assert!(dbg!(result).is_ok());
    }
}
