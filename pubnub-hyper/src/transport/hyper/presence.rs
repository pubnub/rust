//! Presence.

use super::pubsub::inject_subscribe_to;
use super::util::{build_uri, handle_json_response, json_as_array, json_as_object};
use super::{error, Hyper};
use crate::core::data::{presence, request, response};
use crate::core::json;
use crate::core::TransportService;
use async_trait::async_trait;
use hyper::{Body, Response};
use pubnub_util::uritemplate::{IfEmpty, UriTemplate};
use std::collections::HashMap;

async fn handle_presence_response(
    response: Response<Body>,
) -> Result<json::JsonValue, error::Error> {
    let presence_data = handle_json_response(response).await?;

    if presence_data["error"] == true {
        let error_message: String = format!("{}", presence_data["message"]);
        return Err(error::Error::Server(error_message));
    }

    Ok(presence_data)
}

trait HereNowParse<T: presence::respond_with::RespondWith> {
    fn parse(&self, _data_json: &json::JsonValue) -> Option<T::Response> {
        unimplemented!("Attempted parsing unsupported type");
    }

    fn parse_global(&self, data_json: &json::JsonValue) -> Option<presence::GlobalInfo<T>> {
        let payload = json_as_object(&data_json["payload"])?;

        let total_channels = payload["total_channels"].as_u64()?;
        let total_occupancy = payload["total_occupancy"].as_u64()?;

        let channels = {
            let channels = json_as_object(&payload["channels"])?;
            let mut values = HashMap::new();
            for (k, v) in channels.iter() {
                let channel_info = self.parse(v)?;
                values.insert(k.parse().ok()?, channel_info);
            }
            values
        };

        Some(presence::GlobalInfo {
            total_channels,
            total_occupancy,
            channels,
        })
    }
}

impl HereNowParse<presence::respond_with::OccupancyOnly> for () {
    fn parse(
        &self,
        data_json: &json::JsonValue,
    ) -> Option<
        <presence::respond_with::OccupancyOnly as presence::respond_with::RespondWith>::Response,
    > {
        let occupancy = data_json["occupancy"].as_u64()?;
        Some(presence::ChannelInfo { occupancy })
    }
}

impl HereNowParse<presence::respond_with::OccupancyAndUUIDs> for () {
    fn parse(
        &self,
        data_json: &json::JsonValue,
    ) -> Option<
        <presence::respond_with::OccupancyAndUUIDs as presence::respond_with::RespondWith>::Response
>{
        let occupancy = data_json["occupancy"].as_u64()?;

        let occupants = {
            let uuids = json_as_array(&data_json["uuids"])?;
            let results: Option<_> = uuids
                .iter()
                .map(|uuid| uuid.as_str().map(Into::into))
                .collect();
            results?
        };

        Some(presence::ChannelInfoWithOccupants {
            occupancy,
            occupants,
        })
    }
}

impl HereNowParse<presence::respond_with::Full> for () {
    fn parse(
        &self,
        data_json: &json::JsonValue,
    ) -> Option<<presence::respond_with::Full as presence::respond_with::RespondWith>::Response>
    {
        let occupancy = data_json["occupancy"].as_u64()?;

        let occupants = {
            let uuids = json_as_array(&data_json["uuids"])?;
            let results: Option<_> = uuids
                .iter()
                .map(|info| {
                    let info = json_as_object(info)?;

                    let uuid = info["uuid"].as_str().map(Into::into)?;
                    let state = info["state"].clone();

                    Some(presence::ChannelOccupantFullDetails { uuid, state })
                })
                .collect();
            results?
        };

        Some(presence::ChannelInfoWithOccupants {
            occupancy,
            occupants,
        })
    }
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

        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v2/presence/sub-key/{sub_key}/channel/{channel}/uuid/{uuid}/data{?channel-group,state}")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .set_list_with_if_empty("channel", channels, IfEmpty::Dash)
                .set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip)
                .set_scalar("uuid", uuid)
                .set_scalar("state", json::stringify(state))
                .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
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

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}/uuid/{uuid}{?channel-group}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list_with_if_empty("channel", channels, IfEmpty::Dash)
        .set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip)
        .set_scalar("uuid", uuid)
        .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let mut data_json = handle_presence_response(response).await?;

        // Parse response.
        Ok(data_json.remove("payload"))
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
            ..
        } = request;

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?disable_uuids=1&state=0{&channel-group}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list_with_if_empty("channel", channels, IfEmpty::Dash)
        .set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip)
        .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value = HereNowParse::<presence::respond_with::OccupancyOnly>::parse(&(), &data_json)
            .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::HereNow<presence::respond_with::OccupancyAndUUIDs>> for Hyper {
    type Response = response::HereNow<presence::respond_with::OccupancyAndUUIDs>;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::HereNow<presence::respond_with::OccupancyAndUUIDs>,
    ) -> Result<Self::Response, Self::Error> {
        let request::HereNow {
            channels,
            channel_groups,
            ..
        } = request;

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?disable_uuids=0&state=0{&channel-group}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list_with_if_empty("channel", channels, IfEmpty::Dash)
        .set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip)
        .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value =
            HereNowParse::<presence::respond_with::OccupancyAndUUIDs>::parse(&(), &data_json)
                .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::HereNow<presence::respond_with::Full>> for Hyper {
    type Response = response::HereNow<presence::respond_with::Full>;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::HereNow<presence::respond_with::Full>,
    ) -> Result<Self::Response, Self::Error> {
        let request::HereNow {
            channels,
            channel_groups,
            ..
        } = request;

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v2/presence/sub-key/{sub_key}/channel/{channel}?disable_uuids=0&state=1{&channel-group}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list_with_if_empty("channel", channels, IfEmpty::Dash)
        .set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip)
        .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value = HereNowParse::<presence::respond_with::Full>::parse(&(), &data_json)
            .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::GlobalHereNow<presence::respond_with::OccupancyOnly>> for Hyper {
    type Response = response::GlobalHereNow<presence::respond_with::OccupancyOnly>;
    type Error = error::Error;

    // Clippy is glitching with `async-trait`.
    #[allow(clippy::used_underscore_binding)]
    async fn call(
        &self,
        _request: request::GlobalHereNow<presence::respond_with::OccupancyOnly>,
    ) -> Result<Self::Response, Self::Error> {
        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v2/presence/sub-key/{sub_key}?disable_uuids=1&state=0")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value =
            HereNowParse::<presence::respond_with::OccupancyOnly>::parse_global(&(), &data_json)
                .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>> for Hyper {
    type Response = response::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>;
    type Error = error::Error;

    // Clippy is glitching with `async-trait`.
    #[allow(clippy::used_underscore_binding)]
    async fn call(
        &self,
        _request: request::GlobalHereNow<presence::respond_with::OccupancyAndUUIDs>,
    ) -> Result<Self::Response, Self::Error> {
        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v2/presence/sub-key/{sub_key}?disable_uuids=0&state=0")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value = HereNowParse::<presence::respond_with::OccupancyAndUUIDs>::parse_global(
            &(),
            &data_json,
        )
        .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::GlobalHereNow<presence::respond_with::Full>> for Hyper {
    type Response = response::GlobalHereNow<presence::respond_with::Full>;
    type Error = error::Error;

    // Clippy is glitching with `async-trait`.
    #[allow(clippy::used_underscore_binding)]
    async fn call(
        &self,
        _request: request::GlobalHereNow<presence::respond_with::Full>,
    ) -> Result<Self::Response, Self::Error> {
        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v2/presence/sub-key/{sub_key}?disable_uuids=0&state=1")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;

        // Parse response.
        let value = HereNowParse::<presence::respond_with::Full>::parse_global(&(), &data_json)
            .ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(value)
    }
}

#[async_trait]
impl TransportService<request::WhereNow> for Hyper {
    type Response = response::WhereNow;
    type Error = error::Error;

    async fn call(&self, request: request::WhereNow) -> Result<Self::Response, Self::Error> {
        let request::WhereNow { uuid } = request;

        // Prepare the URL.
        let path_and_query = UriTemplate::new("/v2/presence/sub-key/{sub_key}/uuid/{uuid}")
            .set_scalar("sub_key", self.subscribe_key.clone())
            .set_scalar("uuid", uuid)
            .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_presence_response(response).await?;
        let err_fn = || error::Error::UnexpectedResponseSchema(data_json.clone());

        // Parse response.
        let channles = {
            let payloads = json_as_object(&data_json["payload"]).ok_or_else(err_fn)?;
            let channels = json_as_array(&payloads["channels"]).ok_or_else(err_fn)?;
            let results: Option<_> = channels
                .iter()
                .map(|val| val.as_str().and_then(|s| s.parse().ok()))
                .collect();
            results.ok_or_else(err_fn)?
        };
        Ok(channles)
    }
}

#[async_trait]
impl TransportService<request::Heartbeat> for Hyper {
    type Response = response::Heartbeat;
    type Error = error::Error;

    async fn call(&self, request: request::Heartbeat) -> Result<Self::Response, Self::Error> {
        let request::Heartbeat {
            heartbeat,
            to,
            state,
            uuid,
        } = request;

        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v2/presence/sub-key/{sub_key}/channel/{channel}/heartbeat{?channel-group,uuid,state,heartbeat}")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .tap(|val| inject_subscribe_to(val, &to))
                .set_scalar("uuid", uuid)
                .set_optional_scalar("heartbeat", heartbeat.map(|e|e.to_string()))
                .set_scalar("state", json::stringify(state))
                .build();
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let _ = handle_presence_response(response).await?;

        Ok(())
    }
}
