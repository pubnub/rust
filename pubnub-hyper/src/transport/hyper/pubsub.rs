//! Publish / subscribe.

use super::util::{build_uri, handle_json_response};
use super::{error, Hyper};
use crate::core::data::{
    message::{Message, Type},
    request, response,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::TransportService;
use crate::encode_json;
use async_trait::async_trait;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use pubnub_util::encoded_channels_list::EncodedChannelsList;

#[async_trait]
impl TransportService<request::Publish> for Hyper {
    type Response = response::Publish;
    type Error = error::Error;

    async fn call(&self, request: request::Publish) -> Result<Self::Response, Self::Error> {
        // Prepare encoded message and channel.
        encode_json!(request.payload => encoded_payload);
        let encoded_channel = utf8_percent_encode(request.channel.as_ref(), NON_ALPHANUMERIC);

        // Prepare the URL.
        let path_and_query = format!(
            "/publish/{pub_key}/{sub_key}/0/{channel}/0/{message}?uuid={uuid}",
            pub_key = self.publish_key,
            sub_key = self.subscribe_key,
            channel = encoded_channel,
            message = encoded_payload,
            uuid = self.uuid,
        );
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_json_response(response).await?;

        // Parse timetoken.
        let timetoken = Timetoken {
            t: data_json[2].as_str().unwrap().parse().unwrap(),
            r: 0, // TODO
        };

        Ok(timetoken)
    }
}

#[async_trait]
impl TransportService<request::Subscribe> for Hyper {
    type Response = response::Subscribe;
    type Error = error::Error;

    async fn call(&self, request: request::Subscribe) -> Result<Self::Response, Self::Error> {
        // TODO: add caching of repeating params to avoid reencoding.

        // Prepare encoded channels and channel_groups.
        let encoded_channels = EncodedChannelsList::from(request.channels);
        let encoded_channel_groups = EncodedChannelsList::from(request.channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/subscribe/{sub_key}/{channels}/0?channel-group={channel_groups}&tt={tt}&tr={tr}&uuid={uuid}",
            sub_key = self.subscribe_key,
            channels = encoded_channels,
            channel_groups = encoded_channel_groups,
            tt = request.timetoken.t,
            tr = request.timetoken.r,
            uuid = self.uuid,
        );
        let url = build_uri(&self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_json_response(response).await?;

        // Parse timetoken.
        let timetoken = Timetoken {
            t: data_json["t"]["t"].as_str().unwrap().parse().unwrap(),
            r: data_json["t"]["r"].as_u32().unwrap_or(0),
        };

        // Parse messages.
        let messages = {
            let result: Option<Vec<_>> = data_json["m"].members().map(parse_message).collect();
            result.ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?
        };

        Ok((messages, timetoken))
    }
}

fn parse_message(message: &json::JsonValue) -> Option<Message> {
    let message = Message {
        message_type: Type::from_json(&message["e"]),
        route: message["b"].as_str().map(|s| s.parse().ok())?,
        channel: message["c"].as_str()?.parse().ok()?,
        json: message["d"].clone(),
        metadata: message["u"].clone(),
        timetoken: Timetoken {
            t: message["p"]["t"].as_str()?.parse().ok()?,
            r: message["p"]["r"].as_u32().unwrap_or(0),
        },
        client: message["i"].as_str().map(str::to_string),
        subscribe_key: message["k"].to_string(),
        flags: message["f"].as_u32().unwrap_or(0),
    };
    Some(message)
}
