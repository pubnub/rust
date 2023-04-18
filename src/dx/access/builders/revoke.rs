//! PAM revoke token module.
//!
//! This module contains `Revoke Token` request builder.

use crate::{
    core::{
        error::PubNubError,
        headers::{APPLICATION_JSON, CONTENT_TYPE},
        Deserializer, Transport, TransportMethod, TransportRequest,
    },
    dx::{access::*, PubNubClient},
};
use derive_builder::Builder;
use urlencoding::encode;

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(vis = "pub(super)"))]
/// The [`RevokeTokenRequestBuilder`] is used to build revoke access token
/// permissions to access specific resource endpoints request that is sent to
/// the [`PubNub`] network.
///
/// This struct used by the [`revoke_token`] method of the [`PubNubClient`].
/// The [`revoke_token`] method is used to revoke access token permissions.
///
/// [`PubNub`]:https://www.pubnub.com/
/// [`revoke_token`]: crate::dx::PubNubClient::revoke_token
pub struct RevokeTokenRequest<T, D>
where
    T: Transport,
    D: for<'de> Deserializer<'de, RevokeTokenResponseBody>,
{
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) pubnub_client: PubNubClient<T>,

    /// Service response deserializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) deserializer: D,

    /// Access token for which permissions should be revoked.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) token: String,
}

/// The [`RevokeTokenRequestWithDeserializerBuilder`] is used to build revoke
/// access token permissions to access specific resource endpoints request that
/// is sent to the [`PubNub`] network.
///
/// This struct used by the [`revoke_token`] method of the [`PubNubClient`] and
/// let specify custom deserializer for [`PubNub`] network response.
/// The [`revoke_token`] method is used to revoke access token permissions.
///
/// [`PubNub`]:https://www.pubnub.com/
#[cfg(not(feature = "serde"))]
pub struct RevokeTokenRequestWithDeserializerBuilder<T>
where
    T: Transport,
{
    /// Current client which can provide transportation to perform the request.
    pub(in crate::dx::access) pubnub_client: PubNubClient<T>,

    /// Access token for which permissions should be revoked.
    pub token: String,
}

impl<T, D> RevokeTokenRequest<T, D>
where
    T: Transport,
    D: for<'de> Deserializer<'de, RevokeTokenResponseBody>,
{
    /// Create transport request from the request builder.
    fn transport_request(&self) -> TransportRequest {
        let sub_key = &self.pubnub_client.config.subscribe_key;

        TransportRequest {
            path: format!("/v3/pam/{sub_key}/grant/{}", encode(&self.token)),
            method: TransportMethod::Delete,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            ..Default::default()
        }
    }
}

impl<T, D> RevokeTokenRequestBuilder<T, D>
where
    T: Transport,
    D: for<'de> Deserializer<'de, RevokeTokenResponseBody>,
{
    /// Build and call request.
    pub async fn execute(self) -> Result<RevokeTokenResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self.build().map_err(|err| match err {
            RevokeTokenRequestBuilderError::UninitializedField(msg) => {
                PubNubError::general_api_error(msg, None)
            }
            RevokeTokenRequestBuilderError::ValidationError(msg) => {
                PubNubError::general_api_error(msg, None)
            }
        })?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = request.deserializer;

        client
            .transport
            .send(transport_request)
            .await?
            .body
            .map(|bytes| deserializer.deserialize(&bytes))
            .map_or(
                Err(PubNubError::general_api_error(
                    "No body in the response!",
                    None,
                )),
                |response_body| {
                    response_body.and_then::<RevokeTokenResult, _>(|body| body.try_into())
                },
            )
    }
}

#[cfg(not(feature = "serde"))]
impl<T> RevokeTokenRequestWithDeserializerBuilder<T>
where
    T: Transport,
{
    /// Add custom deserializer.
    ///
    /// Adds the deserializer to the [`RevokeTokenRequestBuilder`].
    ///
    /// Instance of [`RevokeTokenRequestBuilder`] returned.
    pub fn deserialize_with<D>(self, deserializer: D) -> RevokeTokenRequestBuilder<T, D>
    where
        D: for<'de> Deserializer<'de, RevokeTokenResponseBody>,
    {
        RevokeTokenRequestBuilder {
            pubnub_client: Some(self.pubnub_client),
            token: Some(self.token),
            deserializer: Some(deserializer),
        }
    }
}
