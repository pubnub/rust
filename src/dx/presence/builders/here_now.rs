//! PubNub Here Now module.
//!
//! The [`HereNowRequestBuilder`] lets you make and execute a Here Now request
//! that will associate a user with a channel.

use derive_builder::Builder;

use crate::{
    core::{
        utils::{
            encoding::{url_encoded_channel_groups, url_encoded_channels},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Transport, TransportMethod, TransportRequest,
    },
    dx::{presence::builders, pubnub_client::PubNubClientInstance},
    lib::{
        alloc::{
            format,
            string::{String, ToString},
            vec,
            vec::Vec,
        },
        collections::HashMap,
    },
    presence::result::{HereNowResponseBody, HereNowResult},
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
pub struct HereNowRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T, D>,

    /// Channels for which to retrieve occupancy information.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option, into),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channels: Vec<String>,

    /// Channel groups for which to retrieve occupancy information.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(into, strip_option),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channel_groups: Vec<String>,

    /// Whether to include UUIDs of users subscribed to the channel(s).
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option),
        default = "true"
    )]
    pub(in crate::dx::presence) include_user_id: bool,

    /// Whether to include state information of users subscribed to the channel(s).
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option),
        default = "false"
    )]
    pub(in crate::dx::presence) include_state: bool,
}

impl<T, D> HereNowRequestBuilder<T, D> {
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
            } else {
                Ok(())
            }
        })
    }

    /// Build [`SetStateRequest`] from builder.
    fn request(self) -> Result<HereNowRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> HereNowRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let mut query: HashMap<String, String> = HashMap::new();

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        self.include_state
            .then(|| query.insert("state".into(), "1".into()));

        (!self.include_user_id).then(|| {
            query.insert("disable_uuids".into(), "1".into());
        });

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub-key/{sub_key}/channel/{}",
                url_encoded_channels(&self.channels),
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: None,
        })
    }
}

impl<T, D> HereNowRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<HereNowResult, PubNubError> {
        let name_replacement = self
            .channels
            .as_ref()
            .and_then(|channels| (channels.len() == 1).then(|| channels[0].clone()));

        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<HereNowResponseBody, _, _, _>(&client.transport, deserializer)
            .await
            .map(|mut result: HereNowResult| {
                name_replacement.is_some().then(|| {
                    result.channels[0].name = name_replacement.expect("Cannot be None");
                });

                result
            })
    }
}

#[cfg(feature = "blocking")]
impl<T, D> HereNowRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<HereNowResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<HereNowResponseBody, _, _, _>(&client.transport, deserializer)
    }
}

// TODO: unit tests for all presence requests.
