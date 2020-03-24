//! Message and relevant types.

use super::channel;
use super::timetoken::Timetoken;
use json::JsonValue;

/// # PubNub Message
///
/// This is the message structure yielded by [`Subscription`].
///
/// [`Subscription`]: crate::Subscription
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Enum Type of Message.
    pub message_type: Type,
    /// Wildcard channel or channel group.
    pub route: Option<Route>,
    /// Origin Channel of Message Receipt.
    pub channel: channel::Name,
    /// Decoded JSON Message Payload.
    pub json: JsonValue,
    /// Metadata of Message.
    pub metadata: JsonValue,
    /// Message ID Timetoken.
    pub timetoken: Timetoken,
    /// Issuing client ID.
    pub client: Option<String>,
    /// Subscribe key associated with the message.
    pub subscribe_key: String,
    /// Message flags.
    pub flags: u32,
}

/// Message route.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Route {
    /// Message arrived on a wildcard channel.
    ChannelWildcard(channel::WildcardSpec),
    /// Message arrived on a channel group.
    ChannelGroup(channel::Name),
}

/// # PubNub Message Types
///
/// PubNub delivers multiple kinds of messages. This enumeration describes the various types
/// available.
///
/// The special `Unknown` variant may be delivered as the PubNub service evolves. It allows
/// applications built on the PubNub Rust client to be forward-compatible without requiring a full
/// client upgrade.
#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum Type {
    /// A class message containing arbitrary payload data.
    Publish,
    /// A Lightweight message.
    Signal,
    /// An Objects service event, like space description updated.
    Objects,
    /// A message action event.
    Action,
    /// Presence event from channel (e.g. another client joined).
    Presence,
    /// Unknown type. The value may have special meaning in some contexts.
    Unknown(u32),
}

impl Default for Message {
    #[must_use]
    fn default() -> Self {
        Self {
            message_type: Type::Unknown(0),
            route: None,
            channel: channel::Name::default(),
            json: JsonValue::Null,
            metadata: JsonValue::Null,
            timetoken: Timetoken::default(),
            client: None,
            subscribe_key: String::default(),
            flags: Default::default(),
        }
    }
}
