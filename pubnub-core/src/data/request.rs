//! Types used by [`crate::Transport`].

use crate::data::object::Object;
use crate::data::timetoken::Timetoken;

/// A request to publish a message to a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishV1 {
    /// A channel name to publish the message to.
    pub channel: String,
    /// The body of the message.
    pub payload: Object,
    /// Additional information associated with the message.
    pub meta: Option<Object>,
}

/// Subscribe to messages on channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeV2 {
    /// The channel name(s) you are subscribing to.
    /// Verify that channels are comprised of valid characters.
    /// You may subscribe to mulitple channels using a comma seperator.
    /// If subscribing to no channels (only channel groups),
    /// use a comma char (,) as a placeholder.
    /// You may subscribe to channels, channels & channel groups,
    /// or just channel groups.
    pub channels: Vec<String>,
    /// A timetoken to use.
    /// tt: 0 (zero) for the initial subscribe, or a valid timetoken if
    /// resuming / continuing / fast-forwarding from a previous subscribe flow.
    /// tr: Region as returned from the initial call with tt=0.
    pub timetoken: Timetoken,
}
