//! Types used by [`crate::Transport`].

use crate::data::object::Object;
use crate::data::presence;
use crate::data::timetoken::Timetoken;
use crate::data::uuid::UUID;
use std::marker::PhantomData;

/// A request to publish a message to a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publish {
    /// A channel name to publish the message to.
    pub channel: String,

    /// The body of the message.
    pub payload: Object,

    /// Additional information associated with the message.
    pub meta: Option<Object>,
}

/// Subscribe to messages on channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribe {
    /// The channel names you are subscribing to.
    pub channels: Vec<String>,

    /// The channel group names you are subscribing to.
    pub channel_groups: Vec<String>,

    /// A timetoken to use.
    /// tt: 0 (zero) for the initial subscribe, or a valid timetoken if
    /// resuming / continuing / fast-forwarding from a previous subscribe flow.
    /// tr: Region as returned from the initial call with tt=0.
    pub timetoken: Timetoken,
}

/// Set state for a user for channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetState {
    /// The channel names to set state for.
    pub channels: Vec<String>,

    /// The channel group names to set state for.
    pub channel_groups: Vec<String>,

    /// The User UUID to set state for.
    pub uuid: UUID,

    /// State to set.
    pub state: Object,
}

/// Get state for a user for channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetState {
    /// The channel names to get the state for.
    pub channels: Vec<String>,

    /// The channel group names to get state for.
    pub channel_groups: Vec<String>,

    /// The User UUID to get state for.
    pub uuid: UUID,
}

/// Retrieve UUID and State Information for subscribed devices on a specific
/// channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNow<TRespondWith>
where
    TRespondWith: presence::respond_with::RespondWith,
{
    /// The channel names to get the state for.
    pub channels: Vec<String>,

    /// The channel group names to get state for.
    pub channel_groups: Vec<String>,

    /// Type that specializes the response type.
    pub respond_with: PhantomData<TRespondWith>,
}

/// Retrieve UUID and State Information for subscribed devices on a all
/// channels.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalHereNow<TRespondWith>
where
    TRespondWith: presence::respond_with::RespondWith,
{
    /// Type that specializes the response type.
    pub respond_with: PhantomData<TRespondWith>,
}

/// Get list of channels user is present in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereNow {
    /// The User UUID to get list of channels for.
    pub uuid: UUID,
}
