use std::sync::Arc;
use std::time::{SystemTime, SystemTimeError};

use json::JsonValue;

/// # PubNub Message
///
/// This is the message type yielded by [`Subscription`]. It is a smart pointer, which allows
/// efficient message passing between threads.
///
/// [`Subscription`]: crate::Subscription
pub type Message = Arc<Instance>;

/// # PubNub Message Instance
///
/// This is the internal data structure for messages.
#[derive(Debug, Clone)]
pub struct Instance {
    /// Enum Type of Message.
    pub message_type: Type,
    /// Wildcard channel or channel group.
    pub route: Option<String>,
    /// Origin Channel of Message Receipt.
    pub channel: String,
    /// Decoded JSON Message Payload.
    pub json: JsonValue,
    /// Metadata of Message.
    pub metadata: JsonValue,
    /// Message ID Timetoken.
    pub timetoken: Timetoken,
    /// Issuing client ID.
    pub client: Option<String>,
    /// Subscribe key associated with the message.
    pub subscribe_key: String,
    /// Message flags.
    pub flags: u32,
}

/// # PubNub Message Types
///
/// PubNub delivers multiple kinds of messages. This enumeration describes the various types
/// available.
///
/// The special `Unknown` variant may be delivered as the PubNub service evolves. It allows
/// applications built on the PubNub Rust client to be forward-compatible without requiring a full
/// client upgrade.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    /// A class message containing arbitrary payload data.
    Publish,
    /// A Lightweight message.
    Signal,
    /// An Objects service event, like space description updated.
    Objects,
    /// A message action event.
    Action,
    /// Presence event from channel (e.g. another client joined).
    Presence,
    /// Unknown type. The value may have special meaning in some contexts.
    Unknown(u32),
}

/// # PubNub Timetoken
///
/// This is the timetoken structure that PubNub uses as a stream index. It allows clients to
/// resume streaming from where they left off for added resiliency.
///
/// [`PubNub::publish`] returns a `Timetoken` that can be used as a message identifier.
///
/// [`PubNub::publish`]: crate::pubnub::PubNub::publish
#[derive(Debug, Clone)]
pub struct Timetoken {
    /// Timetoken
    pub t: u64,
    /// Origin region
    pub r: u32,
}

impl Type {
    /// # Create a `MessageType` from an integer
    ///
    /// Subscribe message pyloads include a non-enumerated integer to describe message types. We
    /// instead provide a concrete type, using this function to convert the integer into the
    /// appropriate type.
    #[must_use]
    pub fn from_json(i: &JsonValue) -> Self {
        match i.as_u32().unwrap_or(0) {
            0 => Self::Publish,
            1 => Self::Signal,
            2 => Self::Objects,
            3 => Self::Action,
            i => Self::Unknown(i),
        }
    }
}

impl Default for Instance {
    #[must_use]
    fn default() -> Self {
        Self {
            message_type: Type::Unknown(0),
            route: Option::default(),
            channel: String::default(),
            json: JsonValue::Null,
            metadata: JsonValue::Null,
            timetoken: Timetoken::default(),
            client: Option::default(),
            subscribe_key: String::default(),
            flags: Default::default(),
        }
    }
}

impl Timetoken {
    /// Create a `Timetoken`.
    ///
    /// # Arguments
    ///
    /// - `time` - A [`SystemTime`] representing when the message was received by the PubNub global
    ///   network.
    /// - `region` - An internal region identifier for the originating region.
    ///
    /// `region` may be set to `0` if you have nothing better to use. The combination of a time and
    /// region gives us a vector clock that represents the message origin in spacetime; when and
    /// where the message was created. Using an appropriate `region` is important for delivery
    /// semantics in a global distributed system.
    ///
    /// # Errors
    ///
    /// Returns an error when the input `time` argument cannot be transformed into a duration.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::SystemTime;
    /// use pubnub_core::Timetoken;
    ///
    /// let now = SystemTime::now();
    /// let timetoken = Timetoken::new(now, 0)?;
    /// # Ok::<(), std::time::SystemTimeError>(())
    /// ```
    ///
    /// Note: Hidden from docs because there is currently no need to create a timetoken. This may
    /// change as the public API evolves.
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
