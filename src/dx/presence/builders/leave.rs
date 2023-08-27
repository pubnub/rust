//! # PubNub presence leave module.
//!
//! The [`LeaveRequestBuilder`] lets you make and execute requests that will
//! announce `leave` of `user_id` from provided channels and groups.

use derive_builder::Builder;

use crate::{
    core::{
        utils::{
            encoding::{url_encoded_channel_groups, url_encoded_channels},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Transport, TransportMethod, TransportRequest,
    },
    dx::{
        presence::{
            builders,
            result::{LeaveResponseBody, LeaveResult},
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

/// The [`LeaveRequestBuilder`] is used to build user `leave` announcement
/// request that is sent to the [`PubNub`] network.
///
/// This struct is used by the [`leave`] method of the [`PubNubClient`].
/// The [`leave`] method is used to announce specified `user_id` is leaving
/// provided channels and groups.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::presence)", validate = "Self::validate"),
    no_std
)]
pub struct LeaveRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T, D>,

    /// Channels for announcement.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option, into),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channels: Vec<String>,

    /// Channel groups for announcement.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(into, strip_option),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channel_groups: Vec<String>,

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option, into))]
    /// Identifier for which `leave` in channels and/or channel groups will be
    /// announced.
    pub(in crate::dx::presence) user_id: String,
}

impl<T, D> LeaveRequestBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// presence leave request instance.
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

    /// Build [`LeaveRequest`] from builder.
    fn request(self) -> Result<LeaveRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> LeaveRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("uuid".into(), self.user_id.to_string());

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub_key/{sub_key}/channel/{}/leave",
                url_encoded_channels(&self.channels)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: None,
        })
    }
}

impl<T, D> LeaveRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<LeaveResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send::<LeaveResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }
}

#[allow(dead_code)]
#[cfg(feature = "blocking")]
impl<T, D> LeaveRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<LeaveResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<LeaveResponseBody, _, _, _>(&client.transport, deserializer)
    }
}
