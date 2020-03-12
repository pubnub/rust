use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::str::FromStr;

/// A list of the symbols, prohibited for use in the channel name.
///
/// Event though the [documented prohibited symbols] are more restrictive,
/// practice shows that PubNub network is capable of returning channel names
/// with the symbols that are outside the documented restructions in the
/// response body. To avoid unnesessary errors when receiving messages from
/// the PubNub network, we prohibit less symbols than documented.
///
/// This list may be extended in the future to match the official documentation.
///
/// [documented prohibited symbols]: https://www.pubnub.com/docs/web-javascript/data-streams-publish-and-subscribe#channel_name_design_consideration
pub const PROHIBITED_SYMBOLS: &[char] = &[','];

/// A Channel name.
///
/// This type represents an exact channel (or channel group) name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Name(String);

impl Name {
    fn is_valid(s: &str) -> bool {
        !s.contains(PROHIBITED_SYMBOLS)
    }

    /// Create a new [`Name`] skipping the validity check.
    #[must_use]
    pub fn from_string_unchecked(s: String) -> Self {
        Self(s)
    }
}

impl TryFrom<String> for Name {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !Self::is_valid(&value) {
            return Err(value);
        }
        Ok(Self(value))
    }
}

impl FromStr for Name {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_valid(s) {
            return Err(());
        }
        Ok(Self(s.to_owned()))
    }
}

impl AsRef<String> for Name {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Name> for String {
    fn from(name: Name) -> String {
        name.0
    }
}

#[cfg(test)]
mod tests {
    use super::Name;

    fn is_valid(s: &str) -> bool {
        Name::is_valid(s)
    }

    #[test]
    fn valid() {
        // Spec.
        assert_eq!(is_valid(""), true);
        assert_eq!(is_valid("qwe"), true);
        assert_eq!(is_valid("123"), true);
    }

    #[test]
    fn valid_but_not_officially() {
        // Spec.
        assert_eq!(is_valid("/"), true);
        assert_eq!(is_valid("\\"), true);
        assert_eq!(is_valid("."), true);
        assert_eq!(is_valid("*"), true);
        assert_eq!(is_valid(":"), true);

        // Real world examples.
        assert_eq!(is_valid("a.b"), true);
        assert_eq!(is_valid("a:b"), true);
        assert_eq!(is_valid("a/b"), true);
        assert_eq!(is_valid("\\a"), true);
        assert_eq!(is_valid("channels_.*"), true);
        assert_eq!(is_valid("channels_*"), true);
    }

    #[test]
    fn invalid() {
        // Spec.
        assert_eq!(is_valid(","), false);

        // Real world examples.
        assert_eq!(is_valid("a,b"), false);
    }
}
