//! Subscriptions module configuration.
//!
//! This module contains [`SubscriptionConfiguration`] which allow user to
//! configure subscription module components.

use crate::{
    core::Deserializer,
    dx::subscribe::{result::SubscribeResponseBody, SubscriptionManager},
    lib::{
        alloc::sync::Arc,
        core::{
            fmt::{Debug, Formatter, Result},
            ops::{Deref, DerefMut},
        },
    },
};

/// Subscription module configuration.
#[derive(Debug)]
pub(crate) struct SubscriptionConfiguration {
    pub(crate) inner: Arc<SubscriptionConfigurationRef>,
}

impl Deref for SubscriptionConfiguration {
    type Target = SubscriptionConfigurationRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubscriptionConfiguration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).expect("Subscription stream is not unique")
    }
}

impl Clone for SubscriptionConfiguration {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Subscription module configuration.
pub(crate) struct SubscriptionConfigurationRef {
    /// Subscription manager
    pub subscription_manager: Arc<SubscriptionManager>,

    /// Received data deserializer.
    pub deserializer: Option<Arc<dyn Deserializer<SubscribeResponseBody>>>,
}

impl Debug for SubscriptionConfigurationRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "SubscriptionConfiguration {{ manager: {:?} }}",
            self.subscription_manager
        )
    }
}
