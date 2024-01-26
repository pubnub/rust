//! # PubNub set state module.
//!
//! The [`SetStateRequestBuilder`] lets you make and execute requests that will
//! associate the provided `state` with `user_id` on the provided list of
//! channels and channels in channel groups.

use derive_builder::Builder;

#[cfg(feature = "std")]
use crate::lib::alloc::sync::Arc;

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
            result::{SetStateResponseBody, SetStateResult},
        },
        pubnub_client::PubNubClientInstance,
    },
    lib::{
        alloc::{
            format,
            string::{String, ToString},
            vec,
            vec::Vec,
        },
        collections::HashMap,
    },
};

#[cfg(feature = "std")]
/// Request execution handling closure.
pub(crate) type SetStateExecuteCall = Arc<dyn Fn(Vec<String>, Option<Vec<u8>>)>;

/// The [`SetStateRequestBuilder`] is used to build `user_id` associated state
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
pub struct SetStateRequest<T, D> {
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

    /// A state that should be associated with the `user_id`.
    ///
    /// The `state` object should be a `HashMap` which represents information
    /// that should be associated with `user_id`.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # use pubnub::core::Serialize;
    /// # fn main() {
    /// let state = HashMap::<String, bool>::from([
    ///     ("is_owner".to_string(), false),
    ///     ("is_admin".to_string(), true)
    /// ]).serialize();
    /// # }
    /// ```
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(custom, strip_option),
        default = "None"
    )]
    pub(in crate::dx::presence) state: Option<Vec<u8>>,

    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option, into))]
    /// Identifier for which `state` should be associated for provided list of
    /// channels and groups.
    pub(in crate::dx::presence) user_id: String,

    #[cfg(feature = "std")]
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(custom, strip_option)
    )]
    /// Set presence state request execution callback.
    pub(in crate::dx::presence) on_execute: SetStateExecuteCall,
}

impl<T, D> SetStateRequestBuilder<T, D> {
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
            } else if self.state.is_none() {
                Err("State is missing".into())
            } else {
                Ok(())
            }
        })
    }

    /// Build [`SetStateRequest`] from builder.
    fn request(self) -> Result<SetStateRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> SetStateRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let config = &self.pubnub_client.config;
        let mut query: HashMap<String, String> = HashMap::new();

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        if let Some(state) = self.state.as_ref() {
            let serialized_state =
                String::from_utf8(state.clone()).map_err(|err| PubNubError::Serialization {
                    details: err.to_string(),
                })?;
            query.insert("state".into(), serialized_state);
        }

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub-key/{}/channel/{}/uuid/{}/data",
                &config.subscribe_key,
                url_encoded_channels(&self.channels),
                url_encode_extended(self.user_id.as_bytes(), UrlEncodeExtension::NonChannelPath)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.to_string(), APPLICATION_JSON.to_string())].into(),
            body: None,
            #[cfg(feature = "std")]
            timeout: config.transport.request_timeout,
        })
    }
}

impl<T, D> SetStateRequestBuilder<T, D>
where
    T: Transport + 'static,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<SetStateResult, PubNubError> {
        let request = self.request()?;

        #[cfg(feature = "std")]
        if !request.channels.is_empty() {
            (request.on_execute)(request.channels.clone(), request.state.clone());
        }

        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<SetStateResponseBody, _, _, _>(
                &client.transport,
                deserializer,
                #[cfg(feature = "std")]
                &client.config.transport.retry_configuration,
                #[cfg(feature = "std")]
                &client.runtime,
            )
            .await
    }
}

#[cfg(feature = "blocking")]
impl<T, D> SetStateRequestBuilder<T, D>
where
    T: crate::core::blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<SetStateResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<SetStateResponseBody, _, _, _>(&client.transport, deserializer)
    }
}
