//! Publish / subscribe.

use super::util::json_as_object;
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
            let result: Option<Vec<_>> = data_json["m"]
                .members()
                .map(|message| match json_as_object(message) {
                    Some(message) => parse_message(message).ok(),
                    None => None,
                })
                .collect();
            result.ok_or_else(|| error::Error::UnexpectedResponseSchema(data_json))?
        };

        Ok((messages, timetoken))
    }
}

fn parse_message_type(i: &json::JsonValue) -> Option<message::Type> {
    let i = if i.is_null() { 0 } else { i.as_u32()? };
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

#[derive(Debug, Clone, PartialEq)]
enum ParseMessageError {
    Type,
    Route,
    Channel,
    Timetoken,
    SubscribeKey,
}

fn parse_message(message: &json::object::Object) -> Result<Message, ParseMessageError> {
    let message = Message {
        message_type: parse_message_type(&message["e"]).ok_or(ParseMessageError::Type)?,
        route: parse_message_route(&message["b"]).map_err(|_| ParseMessageError::Route)?,
        channel: message["c"]
            .as_str()
            .ok_or(ParseMessageError::Channel)?
            .parse()
            .map_err(|_| ParseMessageError::Channel)?,
        json: message["d"].clone(),
        metadata: message["u"].clone(),
        timetoken: Timetoken {
            t: message["p"]["t"]
                .as_str()
                .ok_or(ParseMessageError::Timetoken)?
                .parse()
                .map_err(|_| ParseMessageError::Timetoken)?,
            r: message["p"]["r"].as_u32().unwrap_or(0),
        },
        client: message["i"].as_str().map(std::borrow::ToOwned::to_owned),
        subscribe_key: message["k"]
            .as_str()
            .ok_or(ParseMessageError::SubscribeKey)?
            .to_owned(),
        flags: message["f"].as_u32().unwrap_or(0),
    };
    Ok(message)
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

#[cfg(test)]
mod tests {
    use super::super::util::json_as_object;
    use super::parse_message;
    use crate::core::data::{
        message::{self, Message, Route},
        timetoken::Timetoken,
    };

    #[test]
    fn subscribe_parse_message() {
        let string_sample = r#"{"t":{"t":"15850559815683819","r":12},"m":[{"a":"3","f":514,"i":"31257c03-3722-4409-a0ea-e7b072540115","p":{"t":"15850559815660696","r":12},"k":"demo","c":"demo2","d":"Hello, world!","b":"demo2"}]}"#;
        let json_sample = json::parse(string_sample).unwrap();
        let message_json_object = json_as_object(&json_sample["m"][0]).unwrap();

        let actual_message = parse_message(&message_json_object).unwrap();

        let expected_message = Message {
            message_type: message::Type::Publish,
            route: Some(Route::ChannelWildcard("demo2".parse().unwrap())),
            channel: "demo2".parse().unwrap(),
            json: json::from("Hello, world!"),
            metadata: json::Null,
            timetoken: Timetoken {
                t: 15_850_559_815_660_696,
                r: 12,
            },
            client: Some("31257c03-3722-4409-a0ea-e7b072540115".to_owned()),
            subscribe_key: "demo".to_owned(),
            flags: 514,
        };

        assert_eq!(expected_message, actual_message);
    }
}
