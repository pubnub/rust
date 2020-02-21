//! Types used by [`crate::Transport`].

use crate::data::message::Message;
use crate::data::object::Object;
use crate::data::timetoken::Timetoken;

/// A response to a publish request.
pub type Publish = Timetoken;

/// A response to a subscribe request.
pub type Subscribe = (Vec<Message>, Timetoken);

/// A response to a set state request.
pub type SetState = ();

/// A response to a get state request.
pub type GetState = Object;
