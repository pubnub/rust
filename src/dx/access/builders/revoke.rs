//! PAM revoke token module.
//!
//! This module contains `Revoke Token` request builder.

use crate::{
    core::{
        error::PubNubError,
        utils::{
            encoding::url_encode,
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, Transport, TransportMethod, TransportRequest,
    },
    dx::{access::*, pubnub_client::PubNubClientInstance},
    lib::alloc::{format, string::ToString},
};
use derive_builder::Builder;

#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::access)", validate = "Self::validate"),
    no_std
)]
/// The [`RevokeTokenRequestBuilder`] is used to build revoke access token
/// permissions to access specific resource endpoints request that is sent to
/// the [`PubNub`] network.
///
/// This struct used by the [`revoke_token`] method of the [`PubNubClient`].
/// The [`revoke_token`] method is used to revoke access token permissions.
///
/// [`PubNub`]:https://www.pubnub.com/
/// [`revoke_token`]: crate::dx::PubNubClient::revoke_token
/// [`PubNubClient`]: crate::PubNubClient
pub struct RevokeTokenRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) pubnub_client: PubNubClientInstance<T, D>,

    /// Access token for which permissions should be revoked.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) token: String,
}

impl<T, D> RevokeTokenRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::access) fn transport_request(&self) -> TransportRequest {
        let sub_key = &self.pubnub_client.config.subscribe_key;

        TransportRequest {
            path: format!(
                "/v3/pam/{sub_key}/grant/{}",
                url_encode(self.token.as_bytes())
            ),
            method: TransportMethod::Delete,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            ..Default::default()
        }
    }
}

impl<T, D> RevokeTokenRequestBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        builders::validate_configuration(&self.pubnub_client)
    }
}

impl<T, D> RevokeTokenRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<RevokeTokenResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send::<RevokeTokenResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }
}

#[cfg(feature = "blocking")]
impl<T, D> RevokeTokenRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Execute synchronous request and return the result.
    ///
    /// This method is synchronous and will return result which will resolve to
    /// a [`RevokeTokenResult`] or [`PubNubError`].
    ///
    /// # Example
    /// ```no_run
    /// # use pubnub::{PubNubClientBuilder, Keyset};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_blocking_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .revoke_token("p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI")
    ///     .execute_blocking()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn execute_blocking(self) -> Result<RevokeTokenResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<RevokeTokenResponseBody, _, _, _>(&client.transport, deserializer)
    }
}
