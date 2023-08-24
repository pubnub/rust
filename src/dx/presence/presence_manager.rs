//! Presence module configuration.
//!
//! This module contains [`PresenceManager`] which allow user to configure
//! presence / heartbeat module components.

use crate::{
    dx::presence::event_engine::PresenceEventEngine,
    lib::{
        alloc::sync::Arc,
        core::{
            fmt::{Debug, Formatter, Result},
            ops::{Deref, DerefMut},
        },
    },
};

/// Presence module configuration.
#[derive(Debug)]
pub(crate) struct PresenceManager {
    pub(crate) inner: Arc<PresenceManagerRef>,
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

/// Presence module configuration.
pub(crate) struct PresenceManagerRef {
    /// Presence event engine.
    pub event_engine: Arc<PresenceEventEngine>,

    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// nested `HashMap` with values.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # fn main() {
    /// let state = HashMap::<String, HashMap<<String, bool>>>::from([(
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

impl Debug for PresenceManagerRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "PresenceConfiguration {{\n\tevent_engine: {:?}\n}}",
            self.event_engine
        )
    }
}
