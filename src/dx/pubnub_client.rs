//! TODO: Add documentation

use crate::core::Transport;

/// TODO: Add documentation
pub struct PubNubClient<T>
where
    T: Transport,
{
    pub(crate) transport: T,
    pub(crate) next_seqn: u16,
}
