//! TODO: Add documentation

use pubnub_core::Transport;

/// TODO: Add documentation
pub struct PubNubClient<T>
where
    T: Transport,
{
    pub(crate) transport: T,
}
