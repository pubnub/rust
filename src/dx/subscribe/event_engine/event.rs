use crate::dx::subscribe::result::Update;
use crate::{
    core::{event_engine::Event, PubNubError},
    dx::subscribe::SubscriptionCursor,
    lib::alloc::{string::String, vec::Vec},
};

/// Subscription events.
///
/// Subscribe state machine behaviour depends on from external events which it
/// receives.
#[derive(Debug)]
pub(crate) enum SubscribeEvent {
    /// Current list of channels / groups has been changed.
    ///
    /// Emitted when updates list of channels / groups has been passed for
    /// subscription.
    SubscriptionChanged {
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    },

    /// Catching up on updates.
    ///
    /// Emitted when subscription has been called with timetoken (cursor)
    /// starting from which updates should be received.
    SubscriptionRestored {
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
        cursor: SubscriptionCursor,
    },

    /// Handshake completed successfully.
    ///
    /// Emitted when [`PubNub`] network returned timetoken (cursor) which will
    /// be used for subscription loop.
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    HandshakeSuccess { cursor: SubscriptionCursor },

    /// Handshake completed with error.
    ///
    /// Emitted when handshake effect was unable to receive response from
    /// [`PubNub`] network (network issues or permissions).
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    HandshakeFailure { reason: PubNubError },

    /// Receive updates completed successfully.
    ///
    /// Emitted when [`PubNub`] network returned list of real-time updates along
    /// with timetoken (cursor) which will be used for next subscription loop.
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    ReceiveSuccess {
        cursor: SubscriptionCursor,
        messages: Vec<Update>,
    },

    /// Receive updates completed with error.
    ///
    /// Emitted when receive updates effect was unable to receive response from
    /// [`PubNub`] network (network issues or revoked / expired permissions).
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    ReceiveFailure { reason: PubNubError },

    /// Disconnect from [`PubNub`] network.
    ///
    /// Emitted when explicitly requested to stop receiving real-time updates.
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    Disconnect,

    /// Reconnect to [`PubNub`] network.
    ///
    /// Emitted when explicitly requested to restore real-time updates receive.
    ///
    /// [`PubNub`]: https://www.pubnub.com/
    Reconnect { cursor: Option<SubscriptionCursor> },

    /// Unsubscribe from all channels and groups.
    ///
    /// Emitted when explicitly requested by user to leave all channels and
    /// groups.
    UnsubscribeAll,
}

impl Event for SubscribeEvent {
    fn id(&self) -> &str {
        match self {
            Self::SubscriptionChanged { .. } => "SUBSCRIPTION_CHANGED",
            Self::SubscriptionRestored { .. } => "SUBSCRIPTION_RESTORED",
            Self::HandshakeSuccess { .. } => "HANDSHAKE_SUCCESS",
            Self::HandshakeFailure { .. } => "HANDSHAKE_FAILURE",
            Self::ReceiveSuccess { .. } => "RECEIVE_SUCCESS",
            Self::ReceiveFailure { .. } => "RECEIVE_FAILURE",
            Self::Disconnect => "DISCONNECT",
            Self::Reconnect { .. } => "RECONNECT",
            Self::UnsubscribeAll => "UNSUBSCRIBE_ALL",
        }
    }
}
