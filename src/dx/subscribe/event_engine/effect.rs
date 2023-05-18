use crate::{
    core::{event_engine::Effect, PubNubError},
    dx::subscribe::{SubscribeCursor, SubscribeStatus},
};

/// Subscription state machine effects.
pub enum SubscribeEffect {
    /// Initial subscribe effect invocation.
    Handshake {
        /// Optional list of channels.
        ///
        /// List of channels which will be source of real-time updates after
        /// initial subscription completion.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which will be source of real-time updates
        /// after initial subscription completion.
        channel_groups: Option<Vec<String>>,
    },

    /// Retry initial subscribe effect invocation.
    HandshakeReconnect {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed initial
        /// subscription.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// initial subscription.
        channel_groups: Option<Vec<String>>,

        /// Current initial subscribe retry attempt.
        ///
        /// Used to track overall number of initial subscription retry attempts.
        attempts: u8,

        /// Initial subscribe attempt failure reason.
        reason: PubNubError,
    },

    /// Receive updates effect invocation.
    Receive {
        /// Optional list of channels.
        ///
        /// List of channels for which real-time updates will be delivered.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups for which real-time updates will be
        /// delivered.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,
    },

    /// Retry receive updates effect invocation.
    ReceiveReconnect {
        /// Optional list of channels.
        ///
        /// List of channels which has been used during recently failed receive
        /// updates.
        channels: Option<Vec<String>>,

        /// Optional list of channel groups.
        ///
        /// List of channel groups which has been used during recently failed
        /// receive updates.
        channel_groups: Option<Vec<String>>,

        /// Time cursor.
        ///
        /// Cursor used by subscription loop to identify point in time after
        /// which updates will be delivered.
        cursor: SubscribeCursor,

        /// Current receive retry attempt.
        ///
        /// Used to track overall number of receive updates retry attempts.
        attempts: u8,

        /// Receive updates attempt failure reason.
        reason: PubNubError,
    },

    /// Status change notification effect invocation.
    EmitStatus(SubscribeStatus),

    /// Received updates notification effect invocation.
    EmitMessages(Vec<String>),
}

impl Effect for SubscribeEffect {
    fn id(&self) -> String {
        todo!("Identifiers need to be unique, so we won't cancel wrong effect")
    }
    fn run<F>(&self, f: F)
    where
        F: Fn(),
    {
        f();
        todo!()
    }

    fn cancel(&self) {
        todo!()
    }
}
