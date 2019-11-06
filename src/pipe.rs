use std::sync::Arc;

use crate::channel::ChannelTx;
use tokio::sync::{mpsc, Mutex};

pub(crate) type PipeTx = mpsc::Sender<PipeMessage>;
pub(crate) type PipeRx = mpsc::Receiver<PipeMessage>;
pub(crate) type SharedPipe = Arc<Mutex<Option<Pipe>>>;

/// # Bidirectional communication pipe for `SubscribeLoop`
///
/// `PubNub` owns a reference to one end of the pipe for communiating with the `SubscribeLoop`.
/// `Subscription` owns a clone of the sending side of the pipe to the `SubscribeLoop`, but not the
/// receiving side. `SubscribeLoop` is capable of sending messages to `PubNub`, and receiving
/// messages from both `PubNub` and any number of `Subscription` streams.
///
/// See [`PipeMessage`] for more details.
#[derive(Debug)]
pub(crate) struct Pipe {
    pub(crate) tx: PipeTx, // Send-side for bidirectional communication
    pub(crate) rx: PipeRx, // Recv-side for bidirectional communication
}

/// # The kinds of messages allowed to be delivered over a `Pipe`
#[derive(Debug)]
pub(crate) enum PipeMessage {
    /// A stream for a channel or channel group is being dropped.
    ///
    /// Only sent from `Subscription` to `SubscribeLoop`.
    Drop(usize, ListenerType),

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
    #[cfg(test)]
    Exit,
}

/// # Type of listener (a channel or a channel group)
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ListenerType {
    Channel(String), // Channel name
    _Group(String),  // Channel Group name
}
