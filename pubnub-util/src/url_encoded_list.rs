//! Url Encoded List.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

/// Newtype for an encoded list of channels.
///
/// Immutable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlEncodedList(String);

impl UrlEncodedList {
    /// Create a new [`UrlEncodedList`] from an interator of [`String`]
    /// values.
    pub fn from_str_iter<T, I>(iter: I) -> Self
    where
        T: AsRef<str>,
        I: IntoIterator<Item = T>,
    {
        let iter = iter
            .into_iter()
            .map(|item| utf8_percent_encode(item.as_ref(), NON_ALPHANUMERIC).to_string())
            .collect::<Vec<_>>();
        Self(iter.as_slice().join("%2C"))
    }

    /// Consumes [`UrlEncodedList`] and returns a [`String`].
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<T: AsRef<str>, I: IntoIterator<Item = T>> From<I> for UrlEncodedList {
    fn from(vec: I) -> Self {
        Self::from_str_iter(vec.into_iter())
    }
}

impl AsRef<str> for UrlEncodedList {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for UrlEncodedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let list: &[&str] = &[];
        let res = UrlEncodedList::from(list);
        assert_eq!(res.as_ref(), "");
    }

    #[test]
    fn single() {
        let list: &[&str] = &["qwe"];
        let res = UrlEncodedList::from(list);
        assert_eq!(res.as_ref(), "qwe");
    }

    #[test]
    fn two() {
        let list: &[&str] = &["qwe", "rty"];
        let res = UrlEncodedList::from(list);
        assert_eq!(res.as_ref(), "qwe%2Crty");
    }

    #[test]
    fn encodes_properly() {
        let list: &[&str] = &["hello world", "goodbye world"];
        let res = UrlEncodedList::from(list);
        assert_eq!(res.as_ref(), "hello%20world%2Cgoodbye%20world");
    }
}
