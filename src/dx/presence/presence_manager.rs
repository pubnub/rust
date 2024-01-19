//! Presence module configuration.
//!
//! This module contains [`PresenceManager`] which allow user to configure
//! presence / heartbeat module components.

use crate::presence::event_engine::PresenceEffectInvocation;
use crate::{
    dx::presence::event_engine::{PresenceEvent, PresenceEventEngine},
    lib::{
        alloc::{string::String, sync::Arc, vec::Vec},
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
    pub fn new(
        event_engine: Arc<PresenceEventEngine>,
        heartbeat_interval: u64,
        suppress_leave_events: bool,
    ) -> Self {
        Self {
            inner: Arc::new(PresenceManagerRef {
                event_engine,
                heartbeat_interval,
                suppress_leave_events,
            }),
        }
    }

    /// Terminate subscription manager.
    ///
    /// Gracefully terminate all ongoing tasks including detached event engine
    /// loop.
    #[allow(dead_code)]
    pub fn terminate(&self) {
        self.event_engine
            .stop(PresenceEffectInvocation::TerminateEventEngine);
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

    /// `user_id` presence announcement interval.
    heartbeat_interval: u64,

    /// Whether `user_id` leave should be announced or not.
    ///
    /// When set to `true` and `user_id` will unsubscribe, the client wouldn't
    /// announce `leave`, and as a result, there will be no `leave` presence
    /// event generated.
    suppress_leave_events: bool,
}

impl PresenceManagerRef {
    /// Announce `join` for `user_id` on provided channels and groups.
    pub(crate) fn announce_join(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.event_engine.process(&PresenceEvent::Joined {
            heartbeat_interval: self.heartbeat_interval,
            channels,
            channel_groups,
        });
    }

    /// Announce `leave` for `user_id` on provided channels and groups.
    pub(crate) fn announce_left(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.event_engine.process(&PresenceEvent::Left {
            suppress_leave_events: self.suppress_leave_events,
            channels,
            channel_groups,
        })
    }

    /// Announce `leave` while client disconnected.
    pub(crate) fn disconnect(&self) {
        self.event_engine.process(&PresenceEvent::Disconnect);
    }

    /// Announce `join` upon client connection.
    pub(crate) fn reconnect(&self) {
        self.event_engine.process(&PresenceEvent::Reconnect);
    }
}

impl Debug for PresenceManagerRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "PresenceConfiguration {{ event_engine: {:?} }}",
            self.event_engine
        )
    }
}
