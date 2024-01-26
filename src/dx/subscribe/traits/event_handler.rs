use crate::{
    dx::pubnub_client::PubNubClientInstance,
    lib::alloc::sync::Weak,
    subscribe::{event_engine::SubscriptionInput, SubscriptionCursor, Update},
};

pub trait EventHandler<T, D> {
    /// Handles the given events.
    ///
    /// The implementation should identify the intended recipients and let them
    /// know about a fresh, real-time event if it matches the intended entity.
    ///
    /// # Arguments
    ///
    /// * `cursor` - A time cursor for next portion of events.
    /// * `events` - A slice of real-time events from multiplexed subscription.
    fn handle_events(&self, cursor: SubscriptionCursor, events: &[Update]);

    /// Returns a reference to the subscription input associated with this event
    /// handler.
    ///
    /// # Arguments
    ///
    /// * `include_inactive` - Whether _unused_ entities should be included into
    ///   the subscription input or not.
    ///
    /// # Returns
    ///
    /// A reference to the [`SubscriptionInput`] enum variant.
    fn subscription_input(&self, include_inactive: bool) -> SubscriptionInput;

    /// Invalidates the event handler.
    ///
    /// This method is called to invalidate the event handler, causing any
    /// subscriptions it contains to be invalidated as well.
    fn invalidate(&self);

    /// Returns a reference to the ID associated with the underlying handler.
    ///
    /// # Returns
    ///
    /// Event handler unique identifier.
    fn id(&self) -> &String;

    /// [`PubNubClientInstance`] which is backing event handler.
    ///
    /// # Returns
    ///
    /// Reference on the underlying [`PubNubClientInstance`] instance of
    /// [`Subscription`].
    fn client(&self) -> Weak<PubNubClientInstance<T, D>>;
}
