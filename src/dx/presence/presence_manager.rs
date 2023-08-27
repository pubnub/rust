//! Presence module configuration.
//!
//! This module contains [`PresenceManager`] which allow user to configure
//! presence / heartbeat module components.

use crate::{
    dx::presence::event_engine::{PresenceEvent, PresenceEventEngine},
    lib::{
        alloc::sync::Arc,
        core::{
            fmt::{Debug, Formatter, Result},
            ops::{Deref, DerefMut},
        },
    },
};

/// Presence manager.
///
/// [`PubNubClient`] allows to have state associated with `user_id` on provided
/// list of channels and groups.
#[derive(Debug)]
pub(crate) struct PresenceManager {
    pub(crate) inner: Arc<PresenceManagerRef>,
}

impl PresenceManager {
    pub fn new(event_engine: Arc<PresenceEventEngine>, state: Option<Vec<u8>>) -> Self {
        Self {
            inner: Arc::new(PresenceManagerRef {
                event_engine,
                state,
            }),
        }
    }
}

impl Deref for PresenceManager {
    type Target = PresenceManagerRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PresenceManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Presence configuration is not unique.")
    }
}

impl Clone for PresenceManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Presence manager.
///
/// [`PubNubClient`] allows to have state associated with `user_id` on provided
/// list of channels and groups.
pub(crate) struct PresenceManagerRef {
    /// Presence event engine.
    pub event_engine: Arc<PresenceEventEngine>,

    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// nested `HashMap` with values. State with heartbeat can be set **only**
    /// for channels.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # fn main() {
    /// let state = HashMap::<String, HashMap<String, bool>>::from([(
    ///     "announce".into(),
    ///     HashMap::from([
    ///         ("is_owner".into(), false),
    ///         ("is_admin".into(), true)
    ///     ])
    /// )]);
    /// # }
    /// ```
    pub state: Option<Vec<u8>>,
}

impl PresenceManagerRef {
    /// Announce `join` for `user_id` on provided channels and groups.
    #[allow(dead_code)]
    pub(crate) fn announce_join(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.event_engine.process(&PresenceEvent::Joined {
            channels,
            channel_groups,
        })
    }

    /// Announce `leave` for `user_id` on provided channels and groups.
    #[allow(dead_code)]
    pub(crate) fn announce_left(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.event_engine.process(&PresenceEvent::Left {
            channels,
            channel_groups,
        })
    }
}

impl Debug for PresenceManagerRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "PresenceConfiguration {{\n\tevent_engine: {:?}\n}}",
            self.event_engine
        )
    }
}
