//! Subscription types module.

use crate::lib::core::fmt::{Formatter, Result};

/// Time cursor.
///
/// Cursor used by subscription loop to identify point in time after
/// which updates will be delivered.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub struct SubscribeCursor {
    /// PubNub high-precision timestamp.
    ///
    /// Aside of specifying exact time of receiving data / event this token used
    /// to catchup / follow on real-time updates.
    pub timetoken: u64,

    /// Data center region for which `timetoken` has been generated.
    pub region: u32,
}

/// Subscription statuses.
#[derive(Debug, Copy, Clone)]
pub enum SubscribeStatus {
    /// Successfully connected and receiving real-time updates.
    Connected,

    /// Successfully reconnected after real-time updates received has been
    /// stopped.
    Reconnected,

    /// Real-time updates receive stopped.
    Disconnected,
}

impl core::fmt::Display for SubscribeStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Connected => write!(f, "Connected"),
            Self::Reconnected => write!(f, "Reconnected"),
            Self::Disconnected => write!(f, "Disconnected"),
        }
    }
}
