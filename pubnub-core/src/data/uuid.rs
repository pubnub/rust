//! UUID type.
use std::fmt::{self, Display, Formatter};
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

impl From<&str> for UUID {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
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

impl Display for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
