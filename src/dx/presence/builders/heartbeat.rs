//! # PubNub heartbeat module.
//!
//! The [`HeartbeatRequestBuilder`] allows you to create and execute
//! [`HeartbeatRequests`] that will announce the specified presence in the
//! provided channels and groups.

use crate::{
    core::{
        utils::{
            encoding::{url_encoded_channel_groups, url_encoded_channels},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Serialize, Transport, TransportMethod, TransportRequest,
    },
    dx::{
        presence::{
            builders,
            result::{HeartbeatResponseBody, HeartbeatResult},
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

use derive_builder::Builder;

/// `Heartbeat` `state` marker trait.
pub trait HeartbeatState {}

/// Mark two-dimensional `HashMap` as proper `state` value.
///
/// `Heartbeat` `state` request parameter should be map of maps, where keys of
/// the first level is names of channel and groups and values are dictionary
/// with associated data.
impl<V> HeartbeatState for HashMap<String, HashMap<String, V>> {}

#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::presence)", validate = "Self::validate"),
    no_std
)]
/// The [`HeartbeatRequestsBuilder`] is used to build user presence announce
/// endpoint request that is sent to the [`PubNub`] network.
///
/// [`PubNub`]:https://www.pubnub.com/
pub struct HeartbeatRequests<T, U, D>
where
    U: Serialize + HeartbeatState + Clone,
    D: Deserializer<HeartbeatResponseBody>,
{
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T>,

    /// Service response deserializer.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(super) deserializer: D,

    /// Channels for announcement.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(into, strip_option),
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

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option))]
    pub(in crate::dx::presence) state: Option<U>,

    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option),
        default = "300"
    )]
    pub(in crate::dx::presence) heartbeat: u32,

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option))]
    pub(in crate::dx::presence) user_id: String,
}

/// [`HeartbeatRequestsBuilderWithUuid`] is
pub(crate) struct HeartbeatRequestsBuilderWithUuid<T, S>
where
    S: Into<String>,
{
    /// Current client which can provide transportation to perform the request.
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T>,

    pub(in crate::dx::presence) uuid: S,
}

impl<T, U, D> HeartbeatRequestsBuilder<T, U, D>
where
    U: Serialize + HeartbeatState + Clone,
    D: Deserializer<HeartbeatResponseBody>,
{
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// heartbeat request instance.
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

    /// Determine default `user_id`.
    ///
    /// [`HeartbeatRequestsBuilder`] allows `user_id` to be skipped and will be
    /// set to the current client `user_id`.
    fn set_default_user_id(mut self) -> Result<Self, PubNubError> {
        if self.user_id.is_some() {
            return Ok(self);
        }

        self.user_id = match &self.pubnub_client {
            Some(client) => Some(client.config.user_id.to_string()),
            None => {
                return Err(PubNubError::general_api_error(
                    "PubNub client not set",
                    None,
                    None,
                ));
            }
        };

        Ok(self)
    }
}

impl<T, U, D> HeartbeatRequests<T, U, D>
where
    U: Serialize + HeartbeatState + Clone,
    D: Deserializer<HeartbeatResponseBody>,
{
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("heartbeat".into(), self.heartbeat.to_string());
        query.insert("uuid".into(), self.user_id.to_string());

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        if let Some(state) = &self.state {
            let serialized_state =
                String::from_utf8(state.clone().serialize()?).map_err(|err| {
                    PubNubError::Serialization {
                        details: err.to_string(),
                    }
                })?;
            query.insert("state".into(), serialized_state);
        }

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub_key/{sub_key}/channel/{}/heartbeat",
                url_encoded_channels(&self.channels)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: None,
        })
    }
}

impl<T, U, D> HeartbeatRequestsBuilder<T, U, D>
where
    T: Transport,
    U: Serialize + HeartbeatState + Clone,
    D: Deserializer<HeartbeatResponseBody>,
{
    pub async fn execute(self) -> Result<HeartbeatResult, PubNubError> {
        let builder = self.set_default_user_id()?;
        let request = builder
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = request.deserializer;

        let response = client.transport.send(transport_request).await?;
        response
            .clone()
            .body
            .map(|bytes| {
                let deserialize_result = deserializer.deserialize(&bytes);
                if deserialize_result.is_err() && response.status >= 500 {
                    Err(PubNubError::general_api_error(
                        "Unexpected service response",
                        None,
                        Some(Box::new(response.clone())),
                    ))
                } else {
                    deserialize_result
                }
            })
            .map_or(
                Err(PubNubError::general_api_error(
                    "No body in the response!",
                    None,
                    Some(Box::new(response.clone())),
                )),
                |response_body| {
                    response_body.and_then::<HeartbeatResult, _>(|body| {
                        body.try_into().map_err(|response_error: PubNubError| {
                            response_error.attach_response(response)
                        })
                    })
                },
            )
    }
}

#[cfg(feature = "blocking")]
impl<T, U, D> HeartbeatRequestsBuilder<T, U, D>
where
    T: crate::core::blocking::Transport,
    U: Serialize + HeartbeatState + Clone,
    D: Deserializer<HeartbeatResponseBody>,
{
    pub fn execute_blocking(self) -> Result<HeartbeatResult, PubNubError> {
        let builder = self.set_default_user_id()?;
        let request = builder
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = request.deserializer;

        let response = client.transport.send(transport_request)?;
        response
            .body
            .as_ref()
            .map(|bytes| {
                let deserialize_result = deserializer.deserialize(bytes);
                if deserialize_result.is_err() && response.status >= 500 {
                    Err(PubNubError::general_api_error(
                        "Unexpected service response",
                        None,
                        Some(Box::new(response.clone())),
                    ))
                } else {
                    deserialize_result
                }
            })
            .map_or(
                Err(PubNubError::general_api_error(
                    "No body in the response!",
                    None,
                    Some(Box::new(response.clone())),
                )),
                |response_body| {
                    response_body.and_then::<HeartbeatResult, _>(|body| {
                        body.try_into().map_err(|response_error: PubNubError| {
                            response_error.attach_response(response)
                        })
                    })
                },
            )
    }
}

impl<T, S> HeartbeatRequestsBuilderWithUuid<T, S>
where
    S: Into<String>,
{
    pub(crate) fn with_user_id<U, D>(&self, user_id: S) -> HeartbeatRequestsBuilder<T, U, D>
    where
        U: Serialize + HeartbeatState + Clone,
        D: Deserializer<HeartbeatResponseBody>,
    {
        HeartbeatRequestsBuilder {
            pubnub_client: None,
            deserializer: None,
            channels: None,
            channel_groups: None,
            state: None,
            heartbeat: None,
            user_id: None,
        }
    }
}
