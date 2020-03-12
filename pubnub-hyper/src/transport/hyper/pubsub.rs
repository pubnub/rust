//! Publish / subscribe.

use super::util::{build_uri, handle_json_response};
use super::{error, Hyper};
use crate::core::data::{
    message::{self, Message},
    pubsub, request, response,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::TransportService;
use crate::encode_json;
use async_trait::async_trait;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use pubnub_util::url_encoded_list::UrlEncodedList;

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
        let (channel, channel_groups) = process_subscribe_to(&request.to);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/subscribe/{sub_key}/{channels}/0?channel-group={channel_groups}&tt={tt}&tr={tr}&uuid={uuid}",
            sub_key = self.subscribe_key,
            channels = channel,
            channel_groups = channel_groups,
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

fn parse_message_type(i: &json::JsonValue) -> Option<message::Type> {
    let i = i.as_u32()?;
    Some(match i {
        0 => message::Type::Publish,
        1 => message::Type::Signal,
        2 => message::Type::Objects,
        3 => message::Type::Action,
        i => message::Type::Unknown(i),
    })
}

fn parse_message_route(route: &json::JsonValue) -> Result<Option<message::Route>, ()> {
    if route.is_null() {
        return Ok(None);
    }
    let route = route.as_str().ok_or(())?;
    // First try parsing as wildcard, it is more restrictive.
    if let Ok(val) = route.parse() {
        return Ok(Some(message::Route::ChannelWildcard(val)));
    }
    // Then try parsing as regular name for a channel group.
    if let Ok(val) = route.parse() {
        return Ok(Some(message::Route::ChannelGroup(val)));
    }
    Err(())
}

fn parse_message(message: &json::JsonValue) -> Option<Message> {
    let message = Message {
        message_type: parse_message_type(&message["e"])?,
        route: parse_message_route(&message["b"]).ok()?,
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

fn process_subscribe_to(to: &[pubsub::SubscribeTo]) -> (String, String) {
    let channels = to.iter().filter_map(|to| {
        to.as_channel()
            .map(AsRef::<str>::as_ref)
            .or_else(|| to.as_channel_wildcard().map(AsRef::<str>::as_ref))
    });
    let channel_groups = to.iter().filter_map(pubsub::SubscribeTo::as_channel_group);

    let channels = UrlEncodedList::from(channels).into_inner();
    let channels = if channels.is_empty() {
        "-".to_owned()
    } else {
        channels
    };

    (channels, UrlEncodedList::from(channel_groups).into_inner())
}
