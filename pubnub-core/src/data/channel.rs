//! Channel related types.

use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::str::FromStr;

const NAME_PROHIBITED_SYMBOLS: &[char] = &[',', '/', '\\', '.', '*', ':'];

/// Name represents a channel (or a channel group) name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Name(String);

impl Name {
    fn is_valid(s: &str) -> bool {
        !s.contains(NAME_PROHIBITED_SYMBOLS)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_valid() {
        assert_eq!(Name::is_valid(""), true);
        assert_eq!(Name::is_valid("qwe"), true);
        assert_eq!(Name::is_valid("123"), true);
    }

    #[test]
    fn name_invalid() {
        // Spec.
        assert_eq!(Name::is_valid(","), false);
        assert_eq!(Name::is_valid("/"), false);
        assert_eq!(Name::is_valid("\\"), false);
        assert_eq!(Name::is_valid("."), false);
        assert_eq!(Name::is_valid("*"), false);
        assert_eq!(Name::is_valid(":"), false);

        // Real world examples.
        assert_eq!(Name::is_valid("a,b"), false);
        assert_eq!(Name::is_valid("a.b"), false);
        assert_eq!(Name::is_valid("a:b"), false);
        assert_eq!(Name::is_valid("a/b"), false);
        assert_eq!(Name::is_valid("\\a"), false);
        assert_eq!(Name::is_valid("channels_.*"), false);
        assert_eq!(Name::is_valid("channels_*"), false);
    }
}
