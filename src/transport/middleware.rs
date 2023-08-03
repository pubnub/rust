//! PubNub middleware module.
//!
//! This module contains the middleware that is used to add the required query parameters to the requests.
//! The middleware is used to add the `pnsdk`, `uuid`, `instanceid` and `requestid` query parameters to the requests.

#[cfg(feature = "std")]
use crate::{
    core::{utils::encoding::url_encode, TransportMethod},
    lib::{alloc::vec::Vec, collections::HashMap},
};
use crate::{
    core::{
        utils::metadata::{PKG_VERSION, RUSTC_VERSION, SDK_ID, TARGET},
        PubNubError, Transport, TransportRequest, TransportResponse,
    },
    lib::{
        alloc::{
            boxed::Box,
            format,
            string::{String, ToString},
            sync::Arc,
        },
        core::ops::Deref,
    },
};
#[cfg(feature = "std")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "std")]
use hmac::{Hmac, Mac};
#[cfg(feature = "std")]
use sha2::Sha256;
#[cfg(feature = "std")]
use time::OffsetDateTime;
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
pub struct PubNubMiddleware<T> {
    pub(crate) transport: T,
    pub(crate) instance_id: Arc<Option<String>>,
    pub(crate) user_id: Arc<String>,
    pub(crate) auth_key: Option<Arc<String>>,
    pub(crate) auth_token: Arc<spin::RwLock<String>>,
    #[cfg_attr(not(feature = "std"), allow(dead_code))]
    pub(crate) signature_keys: Option<SignatureKeySet>,
}

#[derive(Debug)]
#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) struct SignatureKeySet {
    pub(crate) secret_key: String,
    pub(crate) publish_key: String,
    pub(crate) subscribe_key: String,
}

#[cfg(feature = "std")]
impl SignatureKeySet {
    fn handle_query_params(query_parameters: &HashMap<String, String>) -> String {
        let mut query_params_str = query_parameters
            .iter()
            .map(|(key, value)| format!("{}={}", key, url_encode(value.as_bytes())))
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

impl<T> PubNubMiddleware<T> {
    fn prepare_request(&self, mut req: TransportRequest) -> Result<TransportRequest, PubNubError> {
        req.query_parameters
            .insert("requestid".into(), Uuid::new_v4().to_string());

        req.query_parameters
            .insert("pnsdk".into(), format!("{}/{}", SDK_ID, PKG_VERSION));
        req.query_parameters
            .entry("uuid".into())
            .or_insert(self.user_id.as_ref().into());

        if let Some(instance_id) = self.instance_id.as_deref() {
            req.query_parameters
                .insert("instanceid".into(), instance_id.into());
        }

        // Adding access token or authorization key.
        if !self.auth_token.read().is_empty() {
            req.query_parameters
                .insert("auth".into(), self.auth_token.read().deref().into());
        } else if let Some(auth_key) = self.auth_key.as_deref() {
            req.query_parameters.insert("auth".into(), auth_key.into());
        }

        #[cfg(feature = "std")]
        if let Some(signature_key_set) = &self.signature_keys {
            req.query_parameters.insert(
                "timestamp".into(),
                OffsetDateTime::now_utc().unix_timestamp().to_string(),
            );
            req.query_parameters.insert(
                "signature".into(),
                signature_key_set.calculate_signature(&req),
            );
        }

        req.headers.insert(
            "User-Agent".into(),
            format!("{}/{} {}/{}", RUSTC_VERSION, TARGET, SDK_ID, PKG_VERSION),
        );

        Ok(req)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl<T> Transport for PubNubMiddleware<T>
where
    T: Transport,
{
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        self.prepare_request(req)
            .map(|req| self.transport.send(req))?
            .await
    }
}

#[cfg(feature = "blocking")]
impl<T> crate::core::blocking::Transport for PubNubMiddleware<T>
where
    T: crate::core::blocking::Transport,
{
    fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
        self.prepare_request(req)
            .and_then(|req| self.transport.send(req))
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::TransportResponse;
    #[cfg(feature = "std")]
    use crate::{core::TransportMethod::Get, lib::collections::HashMap};
    use spin::rwlock::RwLock;

    #[tokio::test]
    async fn include_pubnub_metadata() {
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
                    format!("{}/{}", SDK_ID, PKG_VERSION),
                    request.query_parameters.get("pnsdk").unwrap().clone()
                );
                assert!(request.query_parameters.contains_key("requestid"));

                assert_eq!(
                    format!("{}/{} {}/{}", RUSTC_VERSION, TARGET, SDK_ID, PKG_VERSION),
                    request.headers.get("User-Agent").unwrap().clone()
                );

                Ok(TransportResponse::default())
            }
        }

        let middleware = PubNubMiddleware {
            transport: MockTransport,
            instance_id: Arc::new(Some(String::from("instance_id"))),
            user_id: String::from("user_id").into(),
            signature_keys: None,
            auth_token: Arc::new(RwLock::new(String::new())),
            auth_key: None,
        };

        let result = middleware.send(TransportRequest::default()).await;

        assert!(result.is_ok());
    }

    #[cfg(feature = "std")]
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

    #[cfg(feature = "blocking")]
    #[test]
    fn blocking_transport() {
        use crate::core::blocking::Transport;

        #[derive(Default)]
        struct MockTransport;

        impl crate::core::blocking::Transport for MockTransport {
            fn send(&self, request: TransportRequest) -> Result<TransportResponse, PubNubError> {
                assert_eq!(
                    "user_id",
                    request.query_parameters.get("uuid").unwrap().clone()
                );
                assert_eq!(
                    "instance_id",
                    request.query_parameters.get("instanceid").unwrap().clone()
                );
                assert_eq!(
                    format!("{}/{}", SDK_ID, PKG_VERSION),
                    request.query_parameters.get("pnsdk").unwrap().clone()
                );
                assert!(request.query_parameters.contains_key("requestid"));
                Ok(TransportResponse::default())
            }
        }

        let middleware = PubNubMiddleware {
            transport: MockTransport,
            instance_id: Some(String::from("instance_id")).into(),
            user_id: "user_id".to_string().into(),
            signature_keys: None,
            auth_token: Arc::new(RwLock::new(String::new())),
            auth_key: None,
        };

        let result = middleware.send(TransportRequest::default());

        assert!(result.is_ok());
    }
}
