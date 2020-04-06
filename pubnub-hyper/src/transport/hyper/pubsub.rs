//! Publish / subscribe.

use super::util::json_as_object;
use super::util::uritemplate::{IfEmpty, UriTemplate};
use super::util::{build_uri, handle_json_response};
use super::{error, Hyper};
use crate::core::data::{
    message::{self, Message},
    pubsub, request, response,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::TransportService;
use async_trait::async_trait;

#[async_trait]
impl TransportService<request::Publish> for Hyper {
    type Response = response::Publish;
    type Error = error::Error;

    async fn call(&self, request: request::Publish) -> Result<Self::Response, Self::Error> {
        let request::Publish {
            channel,
            payload,
            .. // TODO: use meta
        } = request;

        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/publish/{pub_key}/{sub_key}/0/{channel}/0/{message}{?uuid}")
                .set_scalar("pub_key", self.publish_key.clone())
                .set_scalar("sub_key", self.subscribe_key.clone())
                .set_scalar("channel", channel)
                .set_scalar("message", json::stringify(payload))
                .set_scalar("uuid", self.uuid.clone())
                .build();
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
        let request::Subscribe {
            to,
            timetoken,
            heartbeat,
        } = request;

        // TODO: add caching of repeating params to avoid reencoding.

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v2/subscribe/{sub_key}/{channel}/0{?channel-group,tt,tr,uuid,heartbeat}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .tap(|val| inject_subscribe_to(val, &to))
        .set_scalar("tt", timetoken.t.to_string())
        .set_scalar("tr", timetoken.r.to_string())
        .set_scalar("uuid", self.uuid.clone())
        .set_optional_scalar("heartbeat", heartbeat.map(|e| e.to_string()))
        .build();
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

pub(super) fn inject_subscribe_to(template: &mut UriTemplate, to: &[pubsub::SubscribeTo]) {
    let channels = to.iter().filter_map(|to| {
        to.as_channel()
            .map(AsRef::<str>::as_ref)
            .or_else(|| to.as_channel_wildcard().map(AsRef::<str>::as_ref))
    });
    template.set_list_with_if_empty("channel", channels, IfEmpty::Dash);

    let channel_groups = to
        .iter()
        .filter_map(|to| to.as_channel_group().map(AsRef::<str>::as_ref));
    template.set_list_with_if_empty("channel-group", channel_groups, IfEmpty::Skip);
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
