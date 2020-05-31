//! Types used by [`crate::Transport`].

use super::history;
use crate::data::channel;
use crate::data::object::Object;
use crate::data::pam;
use crate::data::presence;
use crate::data::pubsub;
use crate::data::timetoken::Timetoken;
use crate::data::uuid::UUID;
use std::marker::PhantomData;

/// A request to publish a message to a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publish {
    /// A channel name to publish the message to.
    pub channel: channel::Name,

    /// The body of the message.
    pub payload: Object,

    /// Additional information associated with the message.
    pub meta: Option<Object>,
}

/// Subscribe to messages on channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribe {
    /// The destinations to subscribe to.
    pub to: Vec<pubsub::SubscribeTo>,

    /// A timetoken to use.
    /// tt: 0 (zero) for the initial subscribe, or a valid timetoken if
    /// resuming / continuing / fast-forwarding from a previous subscribe flow.
    /// tr: Region as returned from the initial call with tt=0.
    pub timetoken: Timetoken,

    /// The heartbeat value to send to the PubNub network.
    pub heartbeat: Option<presence::HeartbeatValue>,
}

/// Set state for a user for channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetState {
    /// The channel names to set state for.
    pub channels: Vec<channel::Name>,

    /// The channel group names to set state for.
    pub channel_groups: Vec<channel::Name>,

    /// The User UUID to set state for.
    pub uuid: UUID,

    /// State to set.
    pub state: Object,
}

/// Get state for a user for channels and/or channel groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetState {
    /// The channel names to get the state for.
    pub channels: Vec<channel::Name>,

    /// The channel group names to get state for.
    pub channel_groups: Vec<channel::Name>,

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
    pub channels: Vec<channel::Name>,

    /// The channel group names to get state for.
    pub channel_groups: Vec<channel::Name>,

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

/// Announce a heartbeat.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heartbeat {
    /// The presence timeout period. If `None`, the default value is used.
    pub heartbeat: Option<presence::HeartbeatValue>,

    /// The subscription destinations to announce heartbeat for.
    pub to: Vec<pubsub::SubscribeTo>,

    /// The User UUID to announce subscribtion for.
    pub uuid: UUID,

    /// State to set for channels and channel groups.
    pub state: Object,
}

/// PAMv3 Grant.
pub type Grant = pam::GrantBody;

/// Fetch history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetHistory {
    /// The channel names to get the history for.
    pub channels: Vec<channel::Name>,

    /// The batch history is limited to 500 channels and only the last 25
    /// messages per channel.
    pub max: Option<usize>,

    /// Direction of time traversal. Default is false, which means timeline is
    /// traversed newest to oldest.
    pub reverse: Option<bool>,

    /// If provided, lets you select a "start date", in Timetoken format.
    /// If not provided, it will default to current time.
    /// Page through results by providing a start OR end time token.
    /// Retrieve a slice of the time line by providing both a start AND end time
    /// token.
    /// Start is "exclusive" - that is, the first item returned will be
    /// the one immediately after the start Timetoken value.
    pub start: Option<history::Timetoken>,

    /// If provided, lets you select an "end date", in Timetoken format.
    /// If not provided, it will provide up to the number of messages defined
    /// in the "max" parameter. Page through results by providing a start OR end
    /// time token. Retrieve a slice of the time line by providing both a start
    /// AND end time token.
    /// End is "exclusive" - that is, if a message is associated exactly with
    /// the end Timetoken, it will be included in the result.
    pub end: Option<history::Timetoken>,

    /// Whether to request metadata to be populated in the returned items or
    /// not.
    pub include_metadata: Option<bool>,
}

/// Delete from history.
///
/// Delete API is asynchronously processed. Getting a successful response
/// implies the Delete request has been received and will be processed soon.
///
/// After the delete has been processed, a webhook is called if the subkey
/// specifies one. If multiple (say N) channels are given in the request
/// a single 200 is returned but N webhook calls will be made.
///
/// There is a setting to accept delete from history requests for a key, which
/// you must enable by checking the `Enable Delete-From-History` checkbox in the
/// key settings for your key in the Administration Portal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteHistory {
    /// The channel names to detete history at.
    pub channels: Vec<channel::Name>,

    /// Start time is not inclusive, as in message with timestamp start will not
    /// be deleted.
    pub start: Option<history::Timetoken>,

    /// End time is inclusive, as in message with timestamp end will be deleted.
    pub end: Option<history::Timetoken>,
}

/// Get message counts over a time period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCountsWithTimetoken {
    /// The channel names to get message counts at.
    pub channels: Vec<channel::Name>,

    /// A single timetoken to cover all channels.
    /// Must be greater than zero.
    pub timetoken: history::Timetoken,
}

/// Get message counts over a time period per channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCountsWithChannelTimetokens {
    /// A list of channels with timetokens to get message counts at.
    /// Timetoken value must be non-zero.
    pub channels: Vec<(channel::Name, history::Timetoken)>,
}
