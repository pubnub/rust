//! # ChannelGroup entity module
//!
//! This module contains the [`ChannelGroup`] type, which can be used as a
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
    lib::alloc::{format, sync::Weak, vec, vec::Vec},
    subscribe::{Subscribable, SubscribableType, Subscriber, Subscription, SubscriptionOptions},
};

/// Channel group entity.
///
/// Entity as a first-class citizen provides access to the entity-specific API.
pub struct ChannelGroup<T, D> {
    inner: Arc<ChannelGroupRef<T, D>>,
}

/// Channel group entity reference.
///
/// This struct contains the actual channel group state. It is wrapped in an Arc
/// by [`ChannelGroup`] and uses internal mutability for its internal state.
///
/// Not intended to be used directly. Use [`ChannelGroup`] instead.
#[derive(Debug)]
pub struct ChannelGroupRef<T, D> {
    /// Reference on backing [`PubNubClientInstance`] client.
    ///
    /// Client is used to support entity-specific actions like:
    /// * subscription
    ///
    /// [`PubNubClientInstance`]: PubNubClientInstance
    client: Arc<PubNubClientInstance<T, D>>,

    /// Unique channel group name.
    ///
    /// Channel group names are used by the [`PubNub API`] as a unique
    /// identifier for resources on which a certain operation should be
    /// performed.
    ///
    /// [`PubNub API`]: https://pubnub.com/docs
    pub name: String,

    /// Active subscriptions count.
    ///
    /// Track number of [`Subscription`] which use this entity to receive
    /// real-time updates.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    subscriptions_count: RwLock<usize>,
}

impl<T, D> ChannelGroup<T, D> {
    /// Creates a new instance of a channel group.
    ///
    /// # Arguments
    ///
    /// * `client` - The client instance used to access [`PubNub API`].
    /// * `name` - The name of the channel group.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    pub(crate) fn new<S>(client: &PubNubClientInstance<T, D>, name: S) -> ChannelGroup<T, D>
    where
        S: Into<String>,
    {
        Self {
            inner: Arc::new(ChannelGroupRef {
                client: Arc::new(client.clone()),
                name: name.into(),
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

impl<T, D> Deref for ChannelGroup<T, D> {
    type Target = ChannelGroupRef<T, D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, D> DerefMut for ChannelGroup<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to the ChannelGroup are not allowed")
    }
}

impl<T, D> Clone for ChannelGroup<T, D> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T, D> PartialEq for ChannelGroup<T, D> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl<T, D> From<ChannelGroup<T, D>> for PubNubEntity<T, D> {
    fn from(value: ChannelGroup<T, D>) -> Self {
        PubNubEntity::ChannelGroup(value)
    }
}

impl<T, D> Debug for ChannelGroup<T, D> {
    #[cfg(all(feature = "subscribe", feature = "std"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "ChannelGroup {{ name: {}, subscriptions_count: {} }}",
            self.name,
            self.subscriptions_count()
        )
    }

    #[cfg(not(all(feature = "subscribe", feature = "std")))]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ChannelGroup {{ name: {} }}", self.name)
    }
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> Subscribable<T, D> for ChannelGroup<T, D> {
    fn names(&self, presence: bool) -> Vec<String> {
        let mut names = vec![self.name.clone()];
        presence.then(|| names.push(format!("{}-pnpres", self.name)));

        names
    }

    fn r#type(&self) -> SubscribableType {
        SubscribableType::ChannelGroup
    }

    fn client(&self) -> Weak<PubNubClientInstance<T, D>> {
        Arc::downgrade(&self.client)
    }
}

#[cfg(all(feature = "subscribe", feature = "std"))]
impl<T, D> Subscriber<T, D> for ChannelGroup<T, D>
where
    T: Transport + Send + Sync + 'static,
    D: Deserializer + Send + Sync + 'static,
{
    fn subscription(&self, options: Option<Vec<SubscriptionOptions>>) -> Arc<Subscription<T, D>> {
        Subscription::new(self.client(), self.clone().into(), options)
    }
}
