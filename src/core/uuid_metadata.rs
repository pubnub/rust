//! # UuidMetadata entity module
//!
//! This module contains the [`UuidMetadata`] type, which can be used as a
//! first-class citizen to access the [`PubNub API`].
//!
//! [`PubNub API`]: https://www.pubnub.com/docs

#[cfg(all(feature = "subscribe", feature = "std"))]
use spin::RwLock;

use crate::{
    core::PubNubEntity,
    dx::pubnub_client::PubNubClientInstance,
    lib::{
        alloc::{string::String, sync::Arc},
        core::{
            cmp::PartialEq,
            fmt::{Debug, Formatter, Result},
            ops::{Deref, DerefMut},
        },
    },
};

#[cfg(all(feature = "subscribe", feature = "std"))]
use crate::{
    core::{Deserializer, Transport},
    lib::alloc::{sync::Weak, vec, vec::Vec},
    subscribe::{Subscribable, SubscribableType, Subscriber, Subscription, SubscriptionOptions},
};

/// UUID metadata entity.
///
/// Entity as a first-class citizen provides access to the entity-specific API.
pub struct UuidMetadata<T, D> {
    inner: Arc<UuidMetadataRef<T, D>>,
}

/// UUID metadata entity reference.
///
/// This struct contains the actual UUID metadata state. It is wrapped in an
/// Arc by [`UuidMetadata`] and uses internal mutability for its internal
/// state.
///
/// Not intended to be used directly. Use [`UuidMetadata`] instead.
#[derive(Debug)]
pub struct UuidMetadataRef<T, D> {
    /// Reference on backing [`PubNubClientInstance`] client.
    ///
    /// Client is used to support entity-specific actions like:
    /// * subscription
    ///
    /// [`PubNubClientInstance`]: PubNubClientInstance
    #[allow(unused)]
    client: Arc<PubNubClientInstance<T, D>>,

    /// Unique uuid metadata object identifier.
    ///
    /// Uuid metadata object identifier used by the [`PubNub API`] as unique
    /// resources on which a certain operation should be performed.
    ///
    /// [`PubNub API`]: https://pubnub.com/docs
    pub id: String,

    /// Active subscriptions count.
    ///
    /// Track number of [`Subscription`] which use this entity to receive
    /// real-time updates.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    subscriptions_count: RwLock<usize>,
}

impl<T, D> UuidMetadata<T, D> {
    /// Creates a new instance of an uuid metadata object.
    ///
    /// # Arguments
    ///
    /// * `client` - The client instance used to access [`PubNub API`].
    /// * `id` - The identifier of the uuid metadata object.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub(crate) fn new<S>(client: &PubNubClientInstance<T, D>, id: S) -> UuidMetadata<T, D>
    where
        S: Into<String>,
    {
        Self {
            inner: Arc::new(UuidMetadataRef {
                client: Arc::new(client.clone()),
                id: id.into(),
                #[cfg(all(feature = "subscribe", feature = "std"))]
                subscriptions_count: RwLock::new(0),
            }),
        }
    }

    /// Increase the subscriptions count.
    ///
    /// Increments the value of the subscriptions count by 1.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn increase_subscriptions_count(&self) {
        let mut subscriptions_count_slot = self.subscriptions_count.write();
        *subscriptions_count_slot += 1;
    }

    /// Decrease the subscriptions count.
    ///
    /// Decrements the value of the subscriptions count by 1.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    ///
    /// > As long as entity used by at least one subscription it can't be
    /// > removed from subscription
    /// loop.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn decrease_subscriptions_count(&self) {
        let mut subscriptions_count_slot = self.subscriptions_count.write();
        if *subscriptions_count_slot > 0 {
            *subscriptions_count_slot -= 1;
        }
    }

    /// Current count of subscriptions.
    ///
    /// > This function is only available when both the `subscribe` and `std`
    /// > features are enabled.
    ///
    /// # Returns
    ///
    /// Returns the current count of subscriptions.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn subscriptions_count(&self) -> usize {
        *self.subscriptions_count.read()
    }
}

impl<T, D> Deref for UuidMetadata<T, D> {
    type Target = UuidMetadataRef<T, D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, D> DerefMut for UuidMetadata<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to the UuidMetadata are not allowed")
    }
}

impl<T, D> Clone for UuidMetadata<T, D> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T, D> PartialEq for UuidMetadata<T, D> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<T, D> From<UuidMetadata<T, D>> for PubNubEntity<T, D> {
    fn from(value: UuidMetadata<T, D>) -> Self {
        PubNubEntity::UuidMetadata(value)
    }
}

impl<T, D> Debug for UuidMetadata<T, D> {
    #[cfg(all(feature = "subscribe", feature = "std"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "UuidMetadata {{ id: {}, subscriptions_count: {} }}",
            self.id,
            self.subscriptions_count()
        )
    }

    #[cfg(not(all(feature = "subscribe", feature = "std")))]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "UuidMetadata {{ id: {} }}", self.id)
    }
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> Subscribable<T, D> for UuidMetadata<T, D> {
    fn names(&self, _presence: bool) -> Vec<String> {
        vec![self.id.clone()]
    }

    fn r#type(&self) -> SubscribableType {
        SubscribableType::Channel
    }

    fn client(&self) -> Weak<PubNubClientInstance<T, D>> {
        Arc::downgrade(&self.client)
    }
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> Subscriber<T, D> for UuidMetadata<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn subscription(&self, options: Option<Vec<SubscriptionOptions>>) -> Arc<Subscription<T, D>> {
        Subscription::new(self.client(), self.clone().into(), options)
    }
}
