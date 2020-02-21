//! Types used by [`crate::Transport`].

use crate::data::message::Message;
use crate::data::object::Object;
use crate::data::presence;
use crate::data::timetoken::Timetoken;

/// A response to a publish request.
pub type Publish = Timetoken;

/// A response to a subscribe request.
pub type Subscribe = (Vec<Message>, Timetoken);

/// A response to a set state request.
pub type SetState = ();

/// A response to a get state request.
pub type GetState = Object;

/// A response to a here now request.
pub type HereNow<T: presence::respond_with::RespondWith> = T::Response;

/// A response to a global here now request.
pub type GlobalHereNow<T: presence::respond_with::RespondWith> = presence::GlobalInfo<T>;

/// A response to a where now request.
pub type WhereNow = (); // TODO
