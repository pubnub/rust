use super::subscribe_loop::*;
use crate::message::Message;
use crate::runtime::Runtime;
use futures_util::stream::Stream;
use futures_util::task::{Context, Poll};
use log::debug;
use std::pin::Pin;
use tokio::stream::Stream as TokioStream;

/// # PubNub Subscription
///
/// This is the message stream returned by [`PubNub::subscribe`]. The stream yields [`Message`]
/// items until it is dropped.
///
/// [`PubNub::subscribe`]: crate::pubnub::PubNub::subscribe
#[derive(Debug)]
pub struct Subscription<TRuntime: Runtime> {
    pub(crate) runtime: TRuntime,  // Runtime to use for managing resources
    pub(crate) name: ListenerType, // Channel or Group name
    pub(crate) id: SubscriptionID, // Unique identifier for the listener
    pub(crate) control_tx: ControlTx, // For cleaning up resources at the subscribe loop when dropped
    pub(crate) channel_rx: ChannelRx, // Stream that produces messages
}

/// `Subscription` is a stream.
impl<TRuntime: Runtime> Stream for Subscription<TRuntime> {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        TokioStream::poll_next(Pin::new(&mut self.get_mut().channel_rx), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        TokioStream::size_hint(&self.channel_rx)
    }
}

/// Remove listener from the associated `SubscribeLoop` when the `Subscription` is dropped.
impl<TRuntime: Runtime> Drop for Subscription<TRuntime> {
    fn drop(&mut self) {
        debug!("Dropping Subscription: {:?}", self.name);

        let command = ControlCommand::Drop(self.id, self.name.clone());
        let mut control_tx = self.control_tx.clone();

        // Spawn a future that will send the drop message for us.
        // See: https://boats.gitlab.io/blog/post/poll-drop/
        self.runtime.spawn(async move {
            control_tx
                .send(command)
                .await
                .expect("Unable to send drop message");
        });
    }
}
