use crate::core::data::{
    message::{Message, Type},
    presence, request, response,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::{Transport, TransportService};
use async_trait::async_trait;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
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
        let url = self.build_uri(&path_and_query)?;

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
        let url = self.build_uri(&path_and_query)?;

        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        Ok(data_json)
    }
}

#[async_trait]
impl TransportService<request::HereNow<presence::respond_with::OccupancyOnly>> for Hyper {
    type Response = response::HereNow<presence::respond_with::OccupancyOnly>;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::HereNow<presence::respond_with::OccupancyOnly>,
    ) -> Result<Self::Response, Self::Error> {
        let request::HereNow {
            channels,
            channel_groups,
            respond_with: _,
        } = request;

        let channels = EncodedChannelsList::from(channels);
        let channel_groups = EncodedChannelsList::from(channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?channel-group={channel_group}&disable_uuids=1&state=0",
            sub_key = self.subscribe_key,
            channel = channels,
            channel_group = channel_groups,
        );
        let url = self.build_uri(&path_and_query)?;

        // Send network request
        let response = self.http_client.get(url).await?;
        let _ = handle_presence_response(response).await?;

        todo!()
    }
}

#[async_trait]
impl TransportService<request::HereNow<presence::respond_with::OccupancyAndUUIDs>> for Hyper {
    type Response = response::HereNow<presence::respond_with::OccupancyAndUUIDs>;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::HereNow<presence::respond_with::OccupancyOnly>,
    ) -> Result<Self::Response, Self::Error> {
        let request::HereNow {
            channels,
            channel_groups,
            respond_with: _,
        } = request;

        let channels = EncodedChannelsList::from(channels);
        let channel_groups = EncodedChannelsList::from(channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?channel-group={channel_group}&disable_uuids=0&state=0",
            sub_key = self.subscribe_key,
            channel = channels,
            channel_group = channel_groups,
        );
        let url = self.build_uri(&path_and_query)?;

        // Send network request
        let response = self.http_client.get(url).await?;
        let _ = handle_presence_response(response).await?;

        todo!()
    }
}

#[async_trait]
impl TransportService<request::HereNow<presence::respond_with::Full>> for Hyper {
    type Response = response::HereNow<presence::respond_with::Full>;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::HereNow<presence::respond_with::OccupancyOnly>,
    ) -> Result<Self::Response, Self::Error> {
        let request::HereNow {
            channels,
            channel_groups,
            respond_with: _,
        } = request;

        let channels = EncodedChannelsList::from(channels);
        let channel_groups = EncodedChannelsList::from(channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?channel-group={channel_group}&disable_uuids=0&state=1",
            sub_key = self.subscribe_key,
            channel = channels,
            channel_group = channel_groups,
        );
        let url = self.build_uri(&path_and_query)?;

        // Send network request
        let response = self.http_client.get(url).await?;
        let _ = handle_presence_response(response).await?;

        todo!()
    }
}
