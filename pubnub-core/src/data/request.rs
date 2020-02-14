//! Types used by [`crate::Transport`].

use crate::data::object::Object;
use crate::data::timetoken::Timetoken;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishV1 {
    pub channel: String,
    pub payload: Object,
    pub meta: Option<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeV2 {
    pub channels: Vec<String>,
    pub timetoken: Timetoken,
}
