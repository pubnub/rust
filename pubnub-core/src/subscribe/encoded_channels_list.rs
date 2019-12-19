use super::registry::Registry;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

// TODO: eliminate this after we switch Transport to passing structured values
// instead of the URLs.

/// Newtype for an encoded list of channels.
///
/// Can only be constructed from a Registry and is immutable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EncodedChannelsList(String);

impl<T> From<&Registry<T>> for EncodedChannelsList {
    fn from(registry: &Registry<T>) -> Self {
        Self(
            registry
                .map
                .keys()
                .map(|channel| utf8_percent_encode(channel, NON_ALPHANUMERIC).to_string())
                .collect::<Vec<_>>()
                .as_slice()
                .join("%2C"),
        )
    }
}

impl AsRef<str> for EncodedChannelsList {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for EncodedChannelsList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
