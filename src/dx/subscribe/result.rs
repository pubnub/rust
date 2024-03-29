//! Subscribe result module.
//!
//! This module contains [`SubscribeResult`] type.
//! The [`SubscribeResult`] type is used to represent results of subscribe
//! operation.

use crate::{
    core::{service_response::APIErrorBody, PubNubError, ScalarValue},
    dx::subscribe::{
        types::Message,
        AppContext, File, MessageAction, Presence, {SubscribeMessageType, SubscriptionCursor},
    },
    lib::{
        alloc::{
            boxed::Box,
            string::{String, ToString},
            vec,
            vec::Vec,
        },
        collections::HashMap,
        core::fmt::Debug,
    },
};

/// The result of a subscribe operation.
/// It contains next subscription cursor and list of real-time updates.
#[derive(Debug)]
pub struct SubscribeResult {
    /// Time cursor for subscription loop.
    ///
    /// Next time cursor which can be used to fetch newer updates or
    /// catchup / restore subscription from specific point in time.
    pub cursor: SubscriptionCursor,

    /// Received real-time updates.
    ///
    /// A few more real-time updates are generated by the [`PubNub`] network, in
    /// addition to published messages and signals:
    /// * `presence` – changes in channel's presence or associated user state
    /// * `message actions` – change of action associated with specific message
    /// * `objects` – changes in one of objects or their relationship: `uuid`,
    ///   `channel` or `membership`
    /// * `file` – file sharing updates
    ///
    /// [`PubNub`]:https://www.pubnub.com/
    pub messages: Vec<Update>,
}

/// Real-time update object.
///
/// Each object represent specific real-time event and provide sufficient
/// information about it.
#[derive(Debug, Clone)]
pub enum Update {
    /// Presence change real-time update.
    ///
    /// Payload represents one of the presence types:
    /// * `join` – new user joined the channel
    /// * `leave` – some user left channel
    /// * `timeout` – service didn't notice user for a while
    /// * `interval` – bulk update on `joined`, `left` and `timeout` users.
    /// * `state-change` - some user changed state associated with him on
    ///   channel.
    Presence(Presence),

    /// Object real-time update.
    AppContext(AppContext),

    /// Message's actions real-time update.
    MessageAction(MessageAction),

    /// File sharing real-time update.
    File(File),

    /// Real-time message update.
    Message(Message),

    /// Real-time signal update.
    Signal(Message),
}

/// [`PubNub API`] raw response for subscribe request.
///
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
pub enum SubscribeResponseBody {
    /// This is success response body for subscribe operation in the Subscriber
    /// service.
    /// It contains information about next time cursor and list of updates which
    /// happened since previous time cursor.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "t": {
    ///         "t": "16866076578137008",
    ///         "r": 43
    ///     },
    ///     "m": [
    ///         {
    ///             "a": "1",
    ///             "f": 0,
    ///             "i": "moon",
    ///             "p": {
    ///                 "t": "16866076578137008",
    ///                 "r": 40
    ///             },
    ///             "c": "test_channel",
    ///             "d": "this can be base64 of encrypted message",
    ///             "b": "test_channel"
    ///         },
    ///         {
    ///             "a": "1",
    ///             "f": 0,
    ///             "i": "moon",
    ///             "p": {
    ///                 "t": "16866076578137108",
    ///                 "r": 40
    ///             },
    ///             "c": "test_channel",
    ///             "d": {
    ///                 "sender": "me",
    ///                 "data": {
    ///                     "secret": "here"
    ///                 }
    ///             },
    ///             "b": "test_channel"
    ///         },
    ///         {
    ///             "a": "1",
    ///             "f": 0,
    ///             "i": "moon",
    ///             "p": {
    ///                 "t": "16866076578137208",
    ///                 "r": 40
    ///             },
    ///             "c": "test_channel",
    ///             "d": {
    ///                 "ping_type": "echo",
    ///                 "value": 16
    ///             },
    ///             "b": "test_channel"
    ///         }
    ///     ]
    /// }
    /// ```
    SuccessResponse(APISuccessBody),

    /// This is an error response body for a subscribe operation in the
    /// Subscribe service.
    /// It contains information about the service that provided the response and
    /// details of what exactly was wrong.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "message": "Forbidden",
    ///     "payload": {
    ///         "channels": [
    ///             "test-channel1"
    ///         ],
    ///         "channel-groups": [
    ///             "test-group1"
    ///         ]
    ///     },
    ///     "error": true,
    ///     "service": "Access Manager",
    ///     "status": 403
    /// }
    /// ```
    ErrorResponse(APIErrorBody),
}

/// Content of successful subscribe REST API operation.
///
/// Body contains next subscription cursor and list of receive real-time
/// updates.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct APISuccessBody {
    /// Next subscription cursor.
    ///
    /// The cursor contains information about the start of the next real-time
    /// update timeframe.
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    pub cursor: SubscriptionCursor,

    /// List of updates.
    ///
    /// Contains list of real-time updates received using previous subscription
    /// cursor.
    #[cfg_attr(feature = "serde", serde(rename = "m"))]
    pub messages: Vec<Envelope>,
}

/// Single entry from subscribe response
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Envelope {
    /// Shard number on which the event has been stored.
    #[cfg_attr(feature = "serde", serde(rename = "a"))]
    pub shard: String,

    /// A numeric representation of enabled debug flags.
    #[cfg_attr(feature = "serde", serde(rename = "f"))]
    pub debug_flags: u32,

    /// PubNub defined event type.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "f"),
        serde(default = "Envelope::default_message_type")
    )]
    pub message_type: SubscribeMessageType,

    /// Identifier of client which sent message (set only when [`publish`]
    /// called with `uuid`).
    ///
    /// [`publish`]: crate::dx::publish
    #[cfg_attr(feature = "serde", serde(rename = "i"), serde(default))]
    pub sender: Option<String>,

    /// Sequence number (set only when [`publish`] called with `seqn`).
    ///
    /// [`publish`]: crate::dx::publish
    #[cfg_attr(feature = "serde", serde(rename = "s"), serde(default))]
    pub sequence_number: Option<u32>,

    /// Message "publish" time.
    ///
    /// This is the time when message has been received by [`PubNub`] network.
    ///
    /// [`PubNub`]: https://www.pubnub.com
    #[cfg_attr(feature = "serde", serde(rename = "p"))]
    pub published: SubscriptionCursor,

    /// Name of channel where update received.
    #[cfg_attr(feature = "serde", serde(rename = "c"))]
    pub channel: String,

    /// Event payload.
    ///
    /// Depending from
    #[cfg_attr(feature = "serde", serde(rename = "d"))]
    pub payload: EnvelopePayload,

    /// Actual name of subscription through which event has been delivered.
    ///
    /// [`PubNubClientInstance`] can be used to subscribe to the group of
    /// channels to receive updates and (group name will be set for field).
    /// With this approach there will be no need to separately add *N* number of
    /// channels to [`subscribe`] method call.
    ///
    /// [`subscribe`]: crate::dx::subscribe
    #[cfg_attr(feature = "serde", serde(rename = "b"), serde(default))]
    pub subscription: Option<String>,

    /// User provided message type (set only when [`publish`] called with
    /// `r#type`).
    ///
    /// [`publish`]: crate::dx::publish
    #[cfg_attr(feature = "serde", serde(rename = "mt"), serde(default))]
    pub r#type: Option<String>,

    /// Identifier of space into which message has been published (set only when
    /// [`publish`] called with `space_id`).
    ///
    /// [`publish`]: crate::dx::publish
    #[cfg_attr(feature = "serde", serde(rename = "si"), serde(default))]
    pub space_id: Option<String>,
}

/// Payload of the real-time update.
///
/// Depending from [`Envelope::message_type`] field value payload can been
/// represented in different ways.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
pub enum EnvelopePayload {
    /// Presence change real-time update.
    ///
    /// Payload represents one of the presence types:
    /// * `join` – new user joined the channel
    /// * `leave` – some user left channel
    /// * `timeout` – service didn't notice user for a while
    /// * `interval` – bulk update on `joined`, `left` and `timeout` users.
    /// * `state-change` - some user changed state associated with him on
    ///   channel.
    Presence {
        /// Presence event type.
        action: Option<String>,

        /// Unix timestamp when presence event has been triggered.
        timestamp: usize,

        /// Unique identification of the user for whom the presence event has
        /// been triggered.
        uuid: Option<String>,

        /// The current occupancy after the presence change is updated.
        occupancy: Option<usize>,

        /// The user's state associated with the channel has been updated.
        #[cfg(feature = "serde")]
        data: Option<serde_json::Value>,

        /// The user's state associated with the channel has been updated.
        #[cfg(not(feature = "serde"))]
        data: Option<Vec<u8>>,

        /// The list of unique user identifiers that `joined` the channel since
        /// the last interval presence update.
        join: Option<Vec<String>>,

        /// The list of unique user identifiers that `left` the channel since
        /// the last interval presence update.
        leave: Option<Vec<String>>,

        /// The list of unique user identifiers that `timeout` the channel since
        /// the last interval presence update.
        timeout: Option<Vec<String>>,
    },
    /// Object realtime update.
    Object {
        /// The type of event that happened during the object update.
        ///
        /// Possible values are:
        /// * `update` - object has been updated
        /// * `delete` - object has been removed
        event: String,

        /// Type of object for which update has been generated.
        ///
        /// Possible values are:
        /// * `uuid` - update for user object
        /// * `space` - update for channel object
        /// * `membership` - update for user membership object
        r#type: String,

        /// Information about object for which update has been generated.
        data: ObjectDataBody,

        /// Name of service which generated update for object.
        source: String,

        /// Version of service which generated update for object.
        version: String,
    },

    /// Message action realtime update.
    MessageAction {
        /// The type of event that happened during the message action update.
        ///
        /// Possible values are:
        /// * `added` - action has been added to the message
        /// * `removed` - action has been removed from message
        event: String,

        /// Information about message action for which update has been
        /// generated.
        data: MessageActionDataBody,

        /// Name of service which generated update for message action.
        source: String,

        /// Version of service which generated update for message action.
        version: String,
    },

    /// File message realtime update.
    File {
        /// Message which has been associated with uploaded file.
        message: String,

        /// Information about uploaded file.
        file: FileDataBody,
    },

    /// Real-time message update.
    #[cfg(feature = "serde")]
    Message(serde_json::Value),

    /// Real-time message update.
    #[cfg(not(feature = "serde"))]
    Message(Vec<u8>),
}

/// Information about object for which update has been generated.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
pub enum ObjectDataBody {
    /// `Channel` object update payload body.
    Channel {
        /// Given name of the channel object.
        name: Option<String>,

        /// `Channel` object additional description.
        description: Option<String>,

        /// `Channel` object type information.
        r#type: Option<String>,

        /// `Channel` object current status.
        status: Option<String>,

        /// Unique `channel` object identifier.
        id: String,

        /// Flatten `HashMap` with additional information associated with
        /// `channel` object.
        custom: Option<HashMap<String, ScalarValue>>,

        /// Recent `channel` object modification date.
        updated: String,

        /// Current `channel` object state hash.
        #[cfg_attr(feature = "serde", serde(rename(deserialize = "eTag")))]
        tag: String,
    },

    /// `Uuid` object update payload body.
    Uuid {
        /// Give `uuid` object name.
        name: Option<String>,

        /// Email address associated with `uuid` object.
        email: Option<String>,

        /// `uuid` object identifier in external systems.
        #[cfg_attr(feature = "serde", serde(rename(deserialize = "externalId")))]
        external_id: Option<String>,

        /// `uuid` object external profile URL.
        #[cfg_attr(feature = "serde", serde(rename(deserialize = "profileUrl")))]
        profile_url: Option<String>,

        /// `Uuid` object type information.
        r#type: Option<String>,

        /// `Uuid` object current status.
        status: Option<String>,

        /// Unique `uuid` object identifier.
        id: String,

        /// Flatten `HashMap` with additional information associated with
        /// `uuid` object.
        custom: Option<HashMap<String, ScalarValue>>,

        /// Recent `uuid` object modification date.
        updated: String,

        /// Current `uuid` object state hash.
        #[cfg_attr(feature = "serde", serde(rename(deserialize = "eTag")))]
        tag: String,
    },

    /// `Membership` object update payload body.
    Membership {
        /// `Channel` object within which `uuid` object registered as member.
        channel: Box<ObjectDataBody>,

        /// Flatten `HashMap` with additional information associated with
        /// `membership` object.
        custom: Option<HashMap<String, ScalarValue>>,

        /// Unique identifier of `uuid` object which has relationship with
        /// `channel`.
        uuid: String,

        /// `Membership` object current status.
        status: Option<String>,

        /// Recent `membership` object modification date.
        updated: String,

        /// Current `membership` object state hash.
        #[cfg_attr(feature = "serde", serde(rename(deserialize = "eTag")))]
        tag: String,
    },
}

/// Information about message action for which update has been generated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct MessageActionDataBody {
    /// Timetoken of message for which action has been added / removed.
    #[cfg_attr(feature = "serde", serde(rename(deserialize = "messageTimetoken")))]
    pub message_timetoken: String,

    /// Timetoken of message action which has been added / removed.
    #[cfg_attr(feature = "serde", serde(rename(deserialize = "actionTimetoken")))]
    pub action_timetoken: String,

    /// Message action type.
    pub r#type: String,

    /// Value associated with message action `type`.
    pub value: String,
}

/// Information about uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct FileDataBody {
    /// Unique identifier of uploaded file.
    pub id: String,

    /// Actual name with which file has been stored.
    pub name: String,
}

impl TryFrom<SubscribeResponseBody> for SubscribeResult {
    type Error = PubNubError;

    fn try_from(value: SubscribeResponseBody) -> Result<Self, Self::Error> {
        match value {
            SubscribeResponseBody::SuccessResponse(resp) => {
                let mut messages = Vec::new();
                for message in resp.messages {
                    messages.push(message.try_into()?)
                }

                Ok(SubscribeResult {
                    cursor: resp.cursor,
                    messages,
                })
            }
            SubscribeResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

#[cfg(feature = "serde")]
impl Envelope {
    /// Default message type.
    fn default_message_type() -> SubscribeMessageType {
        SubscribeMessageType::Message
    }
}

#[cfg(feature = "std")]
impl Update {
    /// Name of subscription.
    ///
    /// Name of channel or channel group on which client subscribed and through
    /// which real-time update has been delivered.
    pub(crate) fn subscription(&self) -> String {
        match self {
            Self::Presence(presence) => presence.subscription(),
            Self::AppContext(object) => object.subscription(),
            Self::MessageAction(reaction) => reaction.subscription.clone(),
            Self::File(file) => file.subscription.clone(),
            Self::Message(message) | Self::Signal(message) => message.subscription.clone(),
        }
    }

    /// PubNub high-precision event timestamp.
    ///
    /// # Returns
    ///
    /// Returns time when event has been emitted.
    pub(crate) fn event_timestamp(&self) -> usize {
        match self {
            Self::Presence(presence) => presence.event_timestamp(),
            Self::AppContext(object) => object.event_timestamp(),
            Self::MessageAction(reaction) => reaction.timestamp,
            Self::File(file) => file.timestamp,
            Self::Message(message) | Self::Signal(message) => message.timestamp,
        }
    }
}

impl TryFrom<Envelope> for Update {
    type Error = PubNubError;

    fn try_from(value: Envelope) -> Result<Self, Self::Error> {
        match value.payload {
            EnvelopePayload::Presence { .. } => Ok(Update::Presence(value.try_into()?)),
            EnvelopePayload::Object { .. }
                if matches!(value.message_type, SubscribeMessageType::Object) =>
            {
                Ok(Update::AppContext(value.try_into()?))
            }
            EnvelopePayload::MessageAction { .. }
                if matches!(value.message_type, SubscribeMessageType::MessageAction) =>
            {
                Ok(Update::MessageAction(value.try_into()?))
            }
            EnvelopePayload::File { .. }
                if matches!(value.message_type, SubscribeMessageType::File) =>
            {
                Ok(Update::File(value.try_into()?))
            }
            EnvelopePayload::Message(_) => {
                if matches!(value.message_type, SubscribeMessageType::Message) {
                    Ok(Update::Message(value.try_into()?))
                } else {
                    Ok(Update::Signal(value.try_into()?))
                }
            }
            _ => Err(PubNubError::Deserialization {
                details: "Unable deserialize unknown payload".to_string(),
            }),
        }
    }
}

impl From<EnvelopePayload> for Vec<u8> {
    #[cfg(feature = "serde")]
    fn from(value: EnvelopePayload) -> Self {
        if let EnvelopePayload::Message(payload) = value {
            return serde_json::to_vec(&payload).unwrap_or_default();
        }
        vec![]
    }

    #[cfg(not(feature = "serde"))]
    fn from(value: EnvelopePayload) -> Self {
        if let EnvelopePayload::Message(payload) = value {
            return payload;
        }
        vec![]
    }
}
