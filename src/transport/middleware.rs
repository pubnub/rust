//! PubNub middleware module.
//!
//! This module contains the middleware that is used to add the required query parameters to the requests.
//! The middleware is used to add the `pnsdk`, `uuid`, `instanceid` and `requestid` query parameters to the requests.

use crate::core::{PubNubError, Transport, TransportMethod, TransportRequest, TransportResponse};
use crate::dx::pubnub_client::{SDK_ID, VERSION};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use urlencoding::encode;
use uuid::Uuid;

/// PubNub middleware.
///
/// This middleware is used to add the required query parameters to the requests.
/// The middleware is used to add the `pnsdk`, `uuid`, `instanceid` and `requestid` query parameters to the requests.
///
/// The `pnsdk` query parameter is used to identify the SDK that is used to make the request.
/// The `uuid` query parameter is used to identify the user that is making the request.
/// The `instanceid` query parameter is used to identify the instance of the SDK that is making the request.
/// The `requestid` query parameter is used to identify the request that is being made.
///
/// It is used internally by the [`PubNubClient`].
/// If you are using the [`PubNubClient`] you don't need to use this middleware.
///
/// [`PubNubClient`]: crate::dx::PubNubClient
#[derive(Debug)]
pub struct PubNubMiddleware<'a, T>
where
    T: Transport,
{
    pub(crate) transport: T,
    pub(crate) instance_id: Option<&'a str>,
    pub(crate) user_id: &'a str,
    pub(crate) signature_keys: Option<SignatureKeySet<'a>>,
}

#[derive(Debug)]
pub(crate) struct SignatureKeySet<'a> {
    pub(crate) secret_key: &'a str,
    pub(crate) publish_key: &'a str,
    pub(crate) subscribe_key: &'a str,
}

impl<'a> SignatureKeySet<'a> {
    fn handle_query_params(query_parameters: &HashMap<String, String>) -> String {
        let mut query_params_str = query_parameters
            .iter()
            .map(|(key, value)| format!("{}={}", key, encode(value)))
            .collect::<Vec<String>>();
        query_params_str.sort_unstable();
        query_params_str.join("&")
    }

    fn prepare_signature_v1_input(&self, req: &TransportRequest) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.subscribe_key,
            self.publish_key,
            req.path,
            SignatureKeySet::handle_query_params(&req.query_parameters)
        )
    }

    fn prepare_signature_v2_input_without_body(&self, req: &TransportRequest) -> String {
        format!(
            "{}\n{}\n{}\n{}\n",
            req.method.to_string().to_ascii_uppercase(),
            self.publish_key,
            req.path,
            SignatureKeySet::handle_query_params(&req.query_parameters)
        )
    }

    fn calculate_signature(&self, req: &TransportRequest) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        if req.method == TransportMethod::Post && req.path.starts_with("/publish") {
            let input = self.prepare_signature_v1_input(req);
            mac.update(input.as_bytes());
            let result = mac.finalize();
            general_purpose::URL_SAFE_NO_PAD.encode(result.into_bytes())
        } else {
            let input = self.prepare_signature_v2_input_without_body(req);
            mac.update(input.as_bytes());
            mac.update(req.body.as_deref().unwrap_or_default());
            let result = mac.finalize();
            format!(
                "v2.{}",
                general_purpose::URL_SAFE_NO_PAD.encode(result.into_bytes())
            )
        }
    }
}

#[async_trait::async_trait]
impl<T> Transport for PubNubMiddleware<'_, T>
where
    T: Transport,
{
    async fn send(&self, mut req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        req.query_parameters
            .insert("requestid".into(), Uuid::new_v4().to_string());
        req.query_parameters
            .insert("pnsdk".into(), format!("{}/{}", SDK_ID, VERSION));
        req.query_parameters
            .insert("uuid".into(), self.user_id.to_string());

        if let Some(instance_id) = &self.instance_id {
            req.query_parameters
                .insert("instanceid".into(), instance_id.to_string());
        }

        if let Some(signature_key_set) = &self.signature_keys {
            req.query_parameters.insert(
                "timestamp".into(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| PubNubError::TransportError(&e.to_string()))?
                    .as_secs()
                    .to_string(),
            );
            req.query_parameters.insert(
                "signature".into(),
                signature_key_set.calculate_signature(&req),
            );
        }

        self.transport.send(req).await
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::TransportMethod::Get;
    use crate::core::TransportResponse;
    use std::collections::HashMap;

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
            instance_id: Some(&String::from("instance_id")),
            user_id: "user_id",
            signature_keys: None,
        };

        let result = middleware.send(TransportRequest::default()).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_signature() {
        let signature_key_set = SignatureKeySet {
            secret_key: "secKey".into(),
            publish_key: "pubKey".into(),
            subscribe_key: "".into(),
        };

        let request = TransportRequest {
            path: "/publish/pubKey/subKey/0/my_channel/0/%22hello%21%22".to_string(),
            method: Get,
            body: None,
            query_parameters: HashMap::from([
                ("store".to_string(), "1".to_string()),
                ("ttl".to_string(), "10".to_string()),
                ("uuid".to_string(), "userId".to_string()),
                ("pnsdk".to_string(), "PubNub-Rust".to_string()),
                ("timestamp".to_string(), "1679642098".to_string()),
            ]),
            ..TransportRequest::default()
        };
        let signature = signature_key_set.calculate_signature(&request);
        assert_eq!("v2.AHl5lMpzyT4qcvvlqaszCjTUqU6dPb10a4_XSaYCNIQ", signature);
    }
}
