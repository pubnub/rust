//! History API types.

use super::object::Object;

/// Timetoken type used in history API.
pub type Timetoken = u64;

/// A history item.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// The message payload.
    pub message: Object,

    /// Message timetoken.
    pub timetoken: Timetoken,

    /// The message metadata.
    pub metadata: Object,
}
