//! TODO: Add documentation

use crate::core::Transport;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const SDK_ID: &str = "PubNub-Rust";

/// TODO: Add documentation
pub struct PubNubClient<T>
where
    T: Transport,
{
    pub(crate) transport: T,
    pub(crate) next_seqn: u16,
}
