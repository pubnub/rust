//! Presence API related structs.

/// A presence event received from the PubNub network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Joined
    Join,
    /// Explicitly Left
    Leave,
    /// Implicitly Left
    Timeout,
    /// State has changed
    StateChange,
}
