use std::sync::Arc;

use crate::subscribe::{ChannelTx, SubscriptionID};
use tokio::sync::{mpsc, Mutex};

// TODO: move to `subscribe/control.rs`

// TODO: unexpose.
#[allow(clippy::module_name_repetitions)]
pub type PipeTx = mpsc::Sender<PipeMessage>;
#[allow(clippy::module_name_repetitions)]
pub type PipeRx = mpsc::Receiver<PipeMessage>;

// TODO: unexpose.
#[allow(clippy::module_name_repetitions)]
pub type SharedPipe = Arc<Mutex<Option<Pipe>>>;

/// # Bidirectional communication pipe for `SubscribeLoop`
///
/// `PubNub` owns a reference to one end of the pipe for communicating with the `SubscribeLoop`.
/// `Subscription` owns a clone of the sending side of the pipe to the `SubscribeLoop`, but not the
/// receiving side. `SubscribeLoop` is capable of sending messages to `PubNub`, and receiving
/// messages from both `PubNub` and any number of `Subscription` streams.
///
/// See [`PipeMessage`] for more details.
// TODO: unexpose.
#[derive(Debug)]
pub struct Pipe {
    pub tx: PipeTx, // Send-side for bidirectional communication
    pub rx: PipeRx, // Recv-side for bidirectional communication
}

// TODO: rename to ControlMessage.
/// # The kinds of messages allowed to be delivered over a `Pipe`
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum PipeMessage {
    /// A stream for a channel or channel group is being dropped.
    ///
    /// Only sent from `Subscription` to `SubscribeLoop`.
    Drop(SubscriptionID, ListenerType),

    /// A stream for a channel or channel group is being created.
    ///
    /// Only sent from `PubNub` to `SubscribeLoop`.
    Add(ListenerType, ChannelTx),

    /// The `SubscribeLoop` is ready to receive messages over PubNub.
    ///
    /// Only sent from `SubscribeLoop` to `PubNub`.
    Ready,

    /// Exit the subscribe loop.
    ///
    /// Only sent from `SubscribeLoop` to `PubNub`, and only in unit tests.
    // TODO: re-enable this.
    // #[cfg(test)]
    Exit,
}

/// # Type of listener (a channel or a channel group)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListenerType {
    Channel(String), // Channel name
    _Group(String),  // Channel Group name
}
