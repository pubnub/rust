//! Target and related types.

use super::channel;

/// Standard target represents a channel names list and a channel group names
/// list that together are suitable for use in the API calls.
/// The value of this type is guaranteed to fulfill the standard invariants
/// required by the API calls - that at least one channel or channel group has
/// to be specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standard {
    /// The channel names you are subscribing to.
    channels: Vec<channel::Name>,

    /// The channel group names you are subscribing to.
    channel_groups: Vec<channel::Name>,
}

impl Standard {
    /// Create a standard target from a list of channels and a list of channel
    /// groups.
    ///
    /// # Errors
    ///
    /// Retuns an error when both channels and channel groups lists are empty.
    ///
    pub fn new(
        channels: Vec<channel::Name>,
        channel_groups: Vec<channel::Name>,
    ) -> Result<Self, (Vec<channel::Name>, Vec<channel::Name>)> {
        if channels.is_empty() && channel_groups.is_empty() {
            return Err((channels, channel_groups));
        }

        Ok(Self {
            channels,
            channel_groups,
        })
    }

    /// Extract target value into channels and channel groups.
    #[must_use]
    pub fn into_inner(self) -> (Vec<channel::Name>, Vec<channel::Name>) {
        (self.channels, self.channel_groups)
    }
}

#[cfg(test)]
mod tests {
    use super::Standard;

    #[test]
    fn standard_valid() {
        assert!(Standard::new(vec!["channel".parse().unwrap()], vec![]).is_ok());
        assert!(Standard::new(vec![], vec!["channel_group".parse().unwrap()]).is_ok());
        assert!(Standard::new(
            vec!["channel".parse().unwrap()],
            vec!["channel_group".parse().unwrap()]
        )
        .is_ok());
        assert!(Standard::new(
            vec!["a".parse().unwrap(), "b".parse().unwrap()],
            vec!["a".parse().unwrap(), "b".parse().unwrap()]
        )
        .is_ok());
    }

    #[test]
    fn standard_invalid() {
        assert!(Standard::new(vec![], vec![]).is_err());
    }
}
