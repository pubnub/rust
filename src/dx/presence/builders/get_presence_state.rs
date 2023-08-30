//! # PubNub set state module.
//!
//! The [`GetStateRequestBuilder`] lets you make and execute requests that will
//! associate the provided `state` with `user_id` on the provided list of
//! channels and channels in channel groups.

use derive_builder::Builder;

use crate::{
    core::{
        utils::{
            encoding::{
                url_encode_extended, url_encoded_channel_groups, url_encoded_channels,
                UrlEncodeExtension,
            },
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Transport, TransportMethod, TransportRequest,
    },
    dx::{
        presence::{
            builders,
            result::{GetStateResponseBody, GetStateResult},
        },
        pubnub_client::PubNubClientInstance,
    },
    lib::{
        alloc::{
            string::{String, ToString},
            vec,
        },
        collections::HashMap,
    },
};

/// The [`GetStateRequestBuilder`] is used to build `user_id` associated state
/// update request that is sent to the [`PubNub`] network.
///
/// This struct is used by the [`set_state`] method of the [`PubNubClient`].
/// The [`set_state`] method is used to update state associated with `user_id`
/// on the provided channels and groups.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::presence)", validate = "Self::validate"),
    no_std
)]
pub struct GetStateRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T, D>,

    /// Channels with which state will be associated.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option, into),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channels: Vec<String>,

    /// Channel groups with which state will be associated.
    ///
    /// The specified state will be associated with channels that have been
    /// included in the specified target channel groups.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(into, strip_option),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channel_groups: Vec<String>,

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option, into))]
    /// Identifier for which `state` should be associated for provided list of
    /// channels and groups.
    pub(in crate::dx::presence) user_id: String,
}

impl<T, D> GetStateRequestBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// set state request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        builders::validate_configuration(&self.pubnub_client).and_then(|_| {
            if channels_len == groups_len && channels_len == 0 {
                Err("Either channels or channel groups should be provided".into())
            } else if self.user_id.is_none() {
                Err("User id is missing".into())
            } else {
                Ok(())
            }
        })
    }

    /// Build [`GetStateRequest`] from builder.
    fn request(self) -> Result<GetStateRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> GetStateRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let mut query: HashMap<String, String> = HashMap::new();

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub-key/{sub_key}/channel/{}/uuid/{}",
                url_encoded_channels(&self.channels),
                url_encode_extended(self.user_id.as_bytes(), UrlEncodeExtension::NonChannelPath)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: None,
        })
    }
}

impl<T, D> GetStateRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<GetStateResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send::<GetStateResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }
}

#[allow(dead_code)]
#[cfg(feature = "blocking")]
impl<T, D> GetStateRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<GetStateResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<GetStateResponseBody, _, _, _>(&client.transport, deserializer)
    }
}
