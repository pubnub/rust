use crate::data::channel;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::str::FromStr;

/// A wildcard Channel specifier.
///
/// This type represents a value suitable for use with [wildcard subscribe]
/// feature.
///
/// Currently you can have up to three levels deep with your channel segment
/// hierarchies, `a.b.c`, for example.
///
/// Developers MUST use .* to denote a wildcard subscription. Just having a * at
/// the end of channel names will NOT get translated to .* and will result in a
/// subscribe error.
///
/// [wildcard subscribe]: https://support.pubnub.com/support/solutions/folders/14000109563
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct WildcardSpec(String);

impl WildcardSpec {
    // See https://support.pubnub.com/support/solutions/articles/14000043664-how-many-channel-segments-are-supported-with-wildcard-subscribe-
    fn is_valid(s: &str) -> bool {
        if s.starts_with('.') {
            // Cannot start with the dot.
            return false;
        }

        // A simple, manually implemented, state machine to check the validity.
        {
            let mut was_dot = false;
            let mut was_asterisk = false;
            let mut dots_count = 0;
            for c in s.chars() {
                if was_asterisk {
                    // Asterisk must be last.
                    return false;
                }

                was_asterisk = false;
                if was_dot {
                    if c == '*' {
                        was_asterisk = true;
                    }
                } else if c == '*' {
                    // Asterisk must be prepended by a dot, but this one wasn't.
                    return false;
                }
                let is_dot = c == '.';
                if is_dot {
                    dots_count += 1;
                    if dots_count > 2 {
                        // There must be at most three segments.
                        return false;
                    }
                }
                was_dot = is_dot;
            }
            if was_dot {
                // Dot must be followed by asterisk if it's at the end.
                return false;
            }
        }

        true
    }

    /// Create a new [`WildcardSpec`] skipping the validity check.
    #[must_use]
    pub fn from_string_unchecked(s: String) -> Self {
        Self(s)
    }
}

impl TryFrom<String> for WildcardSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !Self::is_valid(&value) {
            return Err(value);
        }
        Ok(Self(value))
    }
}

impl FromStr for WildcardSpec {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_valid(s) {
            return Err(());
        }
        Ok(Self(s.to_owned()))
    }
}

impl AsRef<String> for WildcardSpec {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for WildcardSpec {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for WildcardSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<channel::Name> for WildcardSpec {
    fn from(name: channel::Name) -> Self {
        // Name is guaranteed to be valid from the wildcard spec perspective.
        Self::from_string_unchecked(name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::WildcardSpec;

    fn is_valid(s: &str) -> bool {
        WildcardSpec::is_valid(s)
    }

    #[test]
    fn valid() {
        assert!(is_valid("stocks.*")); // from https://support.pubnub.com/support/solutions/articles/14000043663-how-do-i-subscribe-to-a-wildcard-channel-
        assert!(is_valid(""));
    }

    #[test]
    fn valid_from_docs() {
        // From https://support.pubnub.com/support/solutions/articles/14000043664-how-many-channel-segments-are-supported-with-wildcard-subscribe-

        assert!(is_valid("a.*"));
        assert!(is_valid("a.b.*"));
        assert!(is_valid("a.b"));
        assert!(is_valid("a.b.c"));

        // Technically speaking, the last two examples are just single channels
        // without any wildcards, but you can subscribe to any of the above
        // forms.
    }

    #[test]
    fn invalid_incorrect_from_docs() {
        // From https://support.pubnub.com/support/solutions/articles/14000043664-how-many-channel-segments-are-supported-with-wildcard-subscribe-

        assert!(!is_valid("*")); // can not wildcard at the top level to subscribe to all channels
        assert!(!is_valid(".*")); // can not start with a .
        assert!(!is_valid("a.*.b")); // * must be at the end
        assert!(!is_valid("a.")); // the . must be followed by a * when it is at the end of the name
        assert!(!is_valid("a*")); // * must always be preceded with a .
        assert!(!is_valid("a*b")); // * must always be preceded with a . and .* must always be at the end

        // NOTE: The above invalid channel names will actually succeed if you
        // attempt to subscribe to them. They will even succeed when you publish
        // to them. But they will operate as literal channel names and not as
        // wildcard channels. So it is highly recommended that you do not use
        // the invalid forms so as not to confuse the intent of the channel
        // names with wildcard behaviors.
    }

    #[test]
    fn invalid_faulty_from_docs() {
        // From https://support.pubnub.com/support/solutions/articles/14000043664-how-many-channel-segments-are-supported-with-wildcard-subscribe-

        // As stated above, there are valid wildcard channel names and invalid
        // wildcard channel names and even the invalid channel names can be
        // successfully subscribed to without error and published to and
        // subscribes to those invalid channel names will succeed. But it only
        // works as a single, literal channel name and with wildcard behaviors.

        // The one exception to this rule is channels with more than two .
        //characters. If you attempt to subscribe to a channel with more than
        // two . characters (more than three segments) it will succeed, but
        // you will not be able to publish to those channels.

        assert!(!is_valid("a.b.c.d")); // too many segments
        assert!(!is_valid("a.b.c.*")); // too many segments

        // If you do attempt to publish to channel names with more than three
        // segments (three or more . delimiters), then you will receive a 400
        // INVALID error response.
    }
}
