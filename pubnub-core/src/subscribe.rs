use std::pin::Pin;

use crate::message::Message;
use crate::pipe::{ListenerType, PipeMessage, PipeTx};
use crate::runtime::Runtime;
use futures_util::stream::Stream;
use futures_util::task::{Context, Poll};
use log::debug;

pub(crate) mod channel;
// mod control;
mod encoded_channels_list;
mod registry;
mod subscribe_loop;
mod subscribe_request;

pub use subscribe_loop::*;

/// # PubNub Subscription
///
/// This is the message stream returned by [`PubNub::subscribe`]. The stream yields [`Message`]
/// items until it is dropped.
///
/// [`PubNub::subscribe`]: crate::pubnub::PubNub::subscribe
#[derive(Debug)]
pub struct Subscription<TRuntime: Runtime> {
    pub(crate) runtime: TRuntime, // Runtime to use for managing resources
    // TODO: unexpose
    pub name: ListenerType,        // Channel or Group name
    pub(crate) id: SubscriptionID, // Unique identifier for the listener
    pub(crate) tx: PipeTx,         // For interrupting the existing subscribe loop when dropped
    pub(crate) channel: ChannelRx, // Stream that produces messages
}

/// `Subscription` is a stream.
impl<TRuntime: Runtime> Stream for Subscription<TRuntime> {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // XXX: Using an undocumented function here because I can't call the poll_next method?
        self.get_mut().channel.poll_recv(cx)
    }
}

/// Remove listener from the associated `SubscribeLoop` when the `Subscription` is dropped.
impl<TRuntime: Runtime> Drop for Subscription<TRuntime> {
    fn drop(&mut self) {
        debug!("Dropping Subscription: {:?}", self.name);

        let msg = PipeMessage::Drop(self.id, self.name.clone());
        let mut tx = self.tx.clone();

        // Spawn a future that will send the drop message for us.
        // See: https://boats.gitlab.io/blog/post/poll-drop/
        self.runtime.spawn(async move {
            tx.send(msg).await.expect("Unable to send drop message");
        });
    }
}
