use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::str::FromStr;

/// A list of the symbols, prohibited for use in the channel name.
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
        assert!(is_valid(""));
        assert!(is_valid("qwe"));
        assert!(is_valid("123"));
    }

    #[test]
    fn valid_but_not_officially() {
        // Spec.
        assert!(is_valid("/"));
        assert!(is_valid("\\"));
        assert!(is_valid("."));
        assert!(is_valid("*"));
        assert!(is_valid(":"));

        // Real world examples.
        assert!(is_valid("a.b"));
        assert!(is_valid("a:b"));
        assert!(is_valid("a/b"));
        assert!(is_valid("\\a"));
        assert!(is_valid("channels_.*"));
        assert!(is_valid("channels_*"));
    }

    #[test]
    fn invalid() {
        // Spec.
        assert!(!is_valid(","));

        // Real world examples.
        assert!(!is_valid("a,b"));
    }
}
