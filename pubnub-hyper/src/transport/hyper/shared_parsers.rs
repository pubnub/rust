use crate::core::data::{
    message::{self, Message},
    timetoken::Timetoken,
};
use crate::core::json;

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

#[derive(Debug, Clone, PartialEq)]
pub enum ParseMessageError {
    Type,
    Route,
    Channel,
    Timetoken,
    SubscribeKey,
}

/// Parse message from a json object.
pub fn parse_message(message: &json::object::Object) -> Result<Message, ParseMessageError> {
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
