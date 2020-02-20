//! UUID type.
use std::ops::Deref;

/// A unique alphanumeric ID for identifying the client to the PubNub Presence
/// System, as well as for PubNub Analytics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UUID(String);

impl From<String> for UUID {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<UUID> for String {
    fn from(value: UUID) -> String {
        value.0
    }
}

impl Deref for UUID {
    type Target = String;

    /// Provides access to the underlying string.
    #[must_use]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
