//! Presence.

use super::util::*;
use super::{error, Hyper};
use crate::core::data::{request, response};
use crate::core::json;
use crate::core::TransportService;
use crate::encode_json;
use async_trait::async_trait;
use hyper::{Body, Response};
use pubnub_util::encoded_channels_list::EncodedChannelsList;

async fn handle_presence_response(
    response: Response<Body>,
) -> Result<json::JsonValue, error::Error> {
    let mut presence_data = handle_json_response(response).await?;

    if presence_data["error"] == true {
        let error_message: String = format!("{}", presence_data["message"]);
        return Err(error::Error::Server(error_message));
    }

    Ok(presence_data.remove("state"))
}

#[async_trait]
impl TransportService<request::SetState> for Hyper {
    type Response = response::SetState;
    type Error = error::Error;

    async fn call(&self, request: request::SetState) -> Result<Self::Response, Self::Error> {
        let request::SetState {
            channels,
            channel_groups,
            uuid,
            state,
        } = request;

        let channels = EncodedChannelsList::from(channels);
        let channel_groups = EncodedChannelsList::from(channel_groups);
        encode_json!(state => state);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}/uuid/{uuid}/data?channel-group={channel_group}&state={state}",
            sub_key = self.subscribe_key,
            channel = channels,
            channel_group = channel_groups,
            uuid = uuid,
            state = state,
        );
        let url = build_uri(&self, &path_and_query)?;

        // Send network request
        let response = self.http_client.get(url).await?;
        let _ = handle_presence_response(response).await?;

        Ok(())
    }
}

#[async_trait]
impl TransportService<request::GetState> for Hyper {
    type Response = response::GetState;
    type Error = error::Error;

    async fn call(&self, request: request::GetState) -> Result<Self::Response, Self::Error> {
        let request::GetState {
            channels,
            channel_groups,
            uuid,
        } = request;

        let channels = EncodedChannelsList::from(channels);
        let channel_groups = EncodedChannelsList::from(channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}/uuid/{uuid}?channel-group={channel_group}",
            sub_key = self.subscribe_key,
            channel = channels,
            channel_group = channel_groups,
            uuid = uuid,
        );
        let url = build_uri(&self, &path_and_query)?;

        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        Ok(data_json)
    }
}
