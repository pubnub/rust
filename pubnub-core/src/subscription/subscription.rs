use super::subscribe_loop::{ChannelRx, ControlCommand, ControlTx, SubscriptionID};
use crate::data::{message::Message, pubsub};
use crate::runtime::Runtime;
use futures_channel::mpsc;
use futures_util::sink::SinkExt;
use futures_util::stream::Stream;
use futures_util::task::{Context, Poll};
use log::debug;
use std::pin::Pin;

/// # Inbound PubNub message stream
///
/// This is the message stream returned by [`PubNub::subscribe`]. The stream yields [`Message`]
/// items until it is dropped.
///
/// [`PubNub::subscribe`]: crate::pubnub::PubNub::subscribe
#[derive(Debug)]
pub struct Subscription<TRuntime: Runtime> {
    pub(crate) runtime: TRuntime,
    // Runtime to use for managing resources
    pub(crate) destination: pubsub::SubscribeTo,
    // Subscription destination
    pub(crate) id: SubscriptionID,
    // Unique identifier for the listener
    pub(crate) control_tx: ControlTx,
    // For cleaning up resources at the subscribe loop when dropped
    pub(crate) channel_rx: ChannelRx, // Stream that produces messages
}

/// `Subscription` is a stream.
impl<TRuntime: Runtime> Stream for Subscription<TRuntime> {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.get_mut().channel_rx), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        Stream::size_hint(&self.channel_rx)
    }
}

impl<TRuntime: Runtime> Subscription<TRuntime> {
    /// Prepare drop command.
    fn drop_command(&self) -> ControlCommand {
        ControlCommand::Drop(self.id, self.destination.clone())
    }
}

/// Remove listener from the associated `SubscribeLoop` when the `Subscription` is dropped.
impl<TRuntime: Runtime> Drop for Subscription<TRuntime> {
    fn drop(&mut self) {
        debug!("Dropping Subscription: {:?}", self.destination);

        let command = self.drop_command();
        let mut control_tx = self.control_tx.clone();

        // Spawn a future that will send the drop message for us.
        // See: https://boats.gitlab.io/blog/post/poll-drop/
        self.runtime.spawn(async move {
            let drop_send_result = control_tx.send(command).await;
            assert!(
                !is_drop_send_result_error(drop_send_result),
                "Unable to unsubscribe"
            );
        });
    }
}

fn is_drop_send_result_error(result: Result<(), mpsc::SendError>) -> bool {
    match result {
        Ok(_) => false, // not an error
        Err(err @ mpsc::SendError { .. }) if err.is_disconnected() => {
            // Control handle is dead, so we can assume loop is dead
            // too, and that we have, therefore, kind of unsubscribed
            // successfully.
            false
        }
        Err(_) => true,
    }
}
