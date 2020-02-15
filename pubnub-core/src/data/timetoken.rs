//! Timetoken type.

use std::time::{SystemTime, SystemTimeError};

/// # PubNub Timetoken
///
/// This is the timetoken structure that PubNub uses as a stream index.
/// It allows clients to resume streaming from where they left off for added
/// resiliency.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Timetoken {
    /// Timetoken
    pub t: u64,
    /// Origin region
    pub r: u32,
}

impl Timetoken {
    /// Create a `Timetoken`.
    ///
    /// # Arguments
    ///
    /// - `time` - A [`SystemTime`] representing when the message was received
    ///            by the PubNub global network.
    /// - `region` - An internal region identifier for the originating region.
    ///
    /// `region` may be set to `0` if you have nothing better to use. The
    /// combination of a time and region gives us a vector clock that represents
    /// the message origin in spacetime; when and where the message was created.
    /// Using an appropriate `region` is important for delivery semantics in a
    /// global distributed system.
    ///
    /// # Errors
    ///
    /// Returns an error when the input `time` argument cannot be transformed
    /// into a duration.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_core::data::timetoken::Timetoken;
    /// use std::time::SystemTime;
    ///
    /// let now = SystemTime::now();
    /// let timetoken = Timetoken::new(now, 0)?;
    /// # Ok::<(), std::time::SystemTimeError>(())
    /// ```
    pub fn new(time: SystemTime, region: u32) -> Result<Self, SystemTimeError> {
        let time = time.duration_since(SystemTime::UNIX_EPOCH)?;
        let secs = time.as_secs();
        let nanos = time.subsec_nanos();

        // Format the timetoken with the appropriate resolution
        let t = (secs * 10_000_000) | (u64::from(nanos) / 100);

        Ok(Self { t, r: region })
    }
}

impl Default for Timetoken {
    #[must_use]
    fn default() -> Self {
        Self {
            t: u64::default(),
            r: u32::default(),
        }
    }
}

impl std::fmt::Display for Timetoken {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{{ t: {}, r: {} }}", self.t, self.r)
    }
}
