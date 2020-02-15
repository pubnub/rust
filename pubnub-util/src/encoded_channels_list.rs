//! Encoded channels list.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

/// Newtype for an encoded list of channels.
///
/// Immutable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedChannelsList(String);

impl EncodedChannelsList {
    /// Create a new [`EncodedChannelsList`] from an interator of [`String`]
    /// values.
    pub fn from_string_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self(
            iter.into_iter()
                .map(|channel| utf8_percent_encode(&channel, NON_ALPHANUMERIC).to_string())
                .collect::<Vec<_>>()
                .as_slice()
                .join("%2C"),
        )
    }
}

impl From<Vec<String>> for EncodedChannelsList {
    fn from(vec: Vec<String>) -> Self {
        Self::from_string_iter(vec.into_iter())
    }
}

impl AsRef<str> for EncodedChannelsList {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for EncodedChannelsList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
