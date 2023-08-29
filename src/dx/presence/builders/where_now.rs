//! PubNub Where Now module.
//!
//! The [`WhereNowRequestBuilder`] lets you make and execute a Here Now request
//! that will associate a user with a channel.

use derive_builder::Builder;

use crate::{
    core::{
        utils::{
            encoding::{url_encode_extended, UrlEncodeExtension},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Transport, TransportMethod, TransportRequest,
    },
    dx::{presence::builders, pubnub_client::PubNubClientInstance},
    lib::collections::HashMap,
    presence::result::{WhereNowResponseBody, WhereNowResult},
};

/// The Here Now request builder.
///
/// Allows you to build a Here Now request that is sent to the [`PubNub`] network.
///
/// This struct is used by the [`here_now`] method of the [`PubNubClient`].
/// The [`here_now`] method is used to acquire information about the current
/// state of a channel.
///
/// [`PubNub`]: https://www.pubnub.com/
#[derive(Builder, Debug)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::presence)", validate = "Self::validate"),
    no_std
)]
pub struct WhereNowRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T, D>,

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option, into))]
    /// Identifier for which `state` should be associated for provided list of
    /// channels and groups.
    pub(in crate::dx::presence) user_id: String,
}

impl<T, D> WhereNowRequestBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// set state request instance.
    fn validate(&self) -> Result<(), String> {
        builders::validate_configuration(&self.pubnub_client).and_then(|_| {
            self.user_id
                .clone()
                .is_some_and(|id| !id.is_empty())
                .then_some(())
                .ok_or_else(|| {
                    "User ID is empty. It should be provided and not empty string.".to_owned()
                })
        })
    }

    /// Build [`SetStateRequest`] from builder.
    fn request(self) -> Result<WhereNowRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> WhereNowRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let sub_key = &self.pubnub_client.config.subscribe_key;

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub-key/{sub_key}/uuid/{}",
                url_encode_extended(self.user_id.as_bytes(), UrlEncodeExtension::NonChannelPath)
            ),
            query_parameters: HashMap::new(),
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: None,
        })
    }
}

impl<T, D> WhereNowRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<WhereNowResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<WhereNowResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }
}

#[cfg(feature = "blocking")]
impl<T, D> WhereNowRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<WhereNowResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<WhereNowResponseBody, _, _, _>(&client.transport, deserializer)
    }
}
