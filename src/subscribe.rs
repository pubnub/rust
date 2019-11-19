use std::pin::Pin;

use crate::channel::{ChannelMap, ChannelRx};
use crate::http::{subscribe_request, HttpClient};
use crate::message::{Message, Timetoken, Type};
use crate::pipe::{ListenerType, Pipe, PipeMessage, PipeTx};
use futures_util::future::FutureExt;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::{Context, Poll};
use log::{debug, error};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

/// # PubNub Subscription
///
/// This is the message stream returned by [`PubNub::subscribe`]. The stream yields [`Message`]
/// items until it is dropped.
///
/// [`PubNub::subscribe`]: crate::pubnub::PubNub::subscribe
#[derive(Debug)]
pub struct Subscription {
    pub(crate) name: ListenerType, // Channel or Group name
    pub(crate) id: usize,          // Unique identifier for the listener
    pub(crate) tx: PipeTx,         // For interrupting the existing subscribe loop when dropped
    pub(crate) channel: ChannelRx, // Stream that produces messages
}

/// # PubNub Subscribe Loop
///
/// Manages state for a subscribe loop. Can be restarted by creating or dropping a `Subscription`.
/// Subscribe loops will stay active until the last `Subscription` is dropped. (Similar to `Rc` or
/// `Arc`.)
#[derive(Debug)]
pub(crate) struct SubscribeLoop {
    pipe: Pipe,               // Bidirectional communication pipe
    client: HttpClient,       // HTTP Client
    origin: String,           // Copy of the PubNub origin domain
    agent: String,            // Copy of the UserAgent
    subscribe_key: String,    // Copy of the PubNub subscribe key
    channels: ChannelMap,     // Client Channels
    groups: ChannelMap,       // Client Channel Groups
    encoded_channels: String, // A cache of all channel names, URI encoded
    encoded_groups: String,   // A cache of all group names, URI encoded
}

/// `Subscription` is a stream.
impl Stream for Subscription {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // XXX: Using an undocumented function here because I can't call the poll_next method?
        self.get_mut().channel.poll_recv(cx)
    }
}

/// Remove listener from the associated `SubscribeLoop` when the `Subscription` is dropped.
impl Drop for Subscription {
    fn drop(&mut self) {
        debug!("Dropping Subscription: {:?}", self.name);

        let msg = PipeMessage::Drop(self.id, self.name.clone());
        let mut tx = self.tx.clone();

        // Spawn a future that will send the drop message for us.
        // See: https://boats.gitlab.io/blog/post/poll-drop/
        tokio::spawn(async move {
            tx.send(msg).await.expect("Unable to send drop message");
        });
    }
}

/// Implements the subscribe loop, which efficiently polls for new messages.
impl SubscribeLoop {
    pub(crate) fn new(
        pipe: Pipe,
        client: HttpClient,
        origin: String,
        agent: String,
        subscribe_key: String,
        channels: ChannelMap,
        groups: ChannelMap,
    ) -> Self {
        let encoded_channels = Self::encode_channels(&channels);
        let encoded_groups = Self::encode_channels(&groups);

        Self {
            pipe,
            client,
            origin,
            agent,
            subscribe_key,
            channels,
            groups,
            encoded_channels,
            encoded_groups,
        }
    }

    /// # Run the subscribe loop
    ///
    /// This consumes `self` to overcome the problem with borrowing async spawned functions. There
    /// is only one way to terminate the subscribe loop, and that is by asking it (nicely) using
    /// the `Pipe` interface.
    pub(crate) async fn run(mut self) {
        debug!("Starting subscribe loop");
        let mut timetoken = Timetoken::default();
        let mut pipe_rx = self.pipe.rx.fuse();

        loop {
            // Construct URI
            // TODO:
            // - auth key
            // - uuid
            // - signatures
            // - channel groups
            // - filters
            let url = format!(
                "https://{origin}/v2/subscribe/{sub_key}/{channels}/0?tt={tt}&tr={tr}",
                origin = self.origin,
                sub_key = self.subscribe_key,
                channels = self.encoded_channels,
                tt = timetoken.t,
                tr = timetoken.r,
            );
            debug!("URL: {}", url);

            // Send network request
            let url = url.parse().expect("Unable to parse URL");
            let response = subscribe_request(&self.client, url).fuse();
            futures_util::pin_mut!(response);

            #[allow(clippy::mut_mut)]
            let (messages, next_timetoken) = futures_util::select! {
                // This is ugly, but necessary. Moving these into a sub-struct might clean it up.
                // See: http://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/
                msg = pipe_rx.next() => if Self::handle_request(
                    &mut self.channels,
                    &mut self.groups,
                    &mut self.encoded_channels,
                    &mut self.encoded_groups,
                    msg,
                ).await {
                    break;
                } else {
                    continue;
                },

                res = response => {
                    if let Ok((messages, next_timetoken)) = res {
                        (messages, next_timetoken)
                    } else {
                        error!("HTTP error: {:?}", res.unwrap_err());
                        continue;
                    }
                }
            };

            // Send ready message when the subscribe loop is capable of receiving messages
            if timetoken.t == 0 {
                if let Err(error) = self.pipe.tx.send(PipeMessage::Ready).await {
                    error!("Error sending ready message: {:?}", error);
                    break;
                }
            }

            // Save Timetoken for next request
            timetoken = next_timetoken;

            debug!("messages: {:?}", messages);
            debug!("timetoken: {:?}", timetoken);

            // Distribute messages to each listener
            for message in messages as Vec<Message> {
                let route = message
                    .route
                    .clone()
                    .unwrap_or_else(|| message.channel.clone());
                debug!("route: {}", route);
                let listeners = self.channels.get_mut(&route).unwrap();
                debug!("Delivering to {} listeners...", listeners.len());
                for channel_tx in listeners.iter_mut() {
                    if let Err(error) = channel_tx.send(message.clone()).await {
                        error!("Delivery error: {:?}", error);
                    }
                }
            }
        }

        debug!("Stopping subscribe loop");

        #[cfg(test)]
        self.pipe
            .tx
            .send(PipeMessage::Exit)
            .await
            .expect("Unable to send exit message");
    }

    /// # Handle a `PipeMessage` request
    ///
    /// This is split out from the `select!` macro used in the `SubscribeLoop`. Debugging complex
    /// code buried within a macro is very painful. So this allows our development experience to be
    /// flexible and the compiler can actually show us useful errors.
    ///
    /// It is an associated function of `SubScribeLoop` because we can't borrow `self` mutably
    /// while `self.client` is borrowed immutably during the long-poll.
    ///
    /// # Returns
    ///
    /// `true` when `SubscribeLoop` needs to be terminated.
    /// `false` when the subscribe loop needs to be restarted.
    pub(crate) async fn handle_request(
        channels: &mut ChannelMap,
        groups: &mut ChannelMap,
        encoded_channels: &mut String,
        encoded_groups: &mut String,
        msg: Option<PipeMessage>,
    ) -> bool {
        debug!("Got request: {:?}", msg);

        // TODO: DRY this code up by implementing add and remove on `ChannelMap`.
        if let Some(request) = msg {
            match request {
                PipeMessage::Drop(id, listener) => {
                    // Remove channel or group
                    match listener {
                        ListenerType::Channel(name) => {
                            // Remove `name` from ChannelMap and re-encode
                            debug!("Removing channel from SubscribeLoop: {}", name);

                            let listeners = channels
                                .get_mut(&name)
                                .expect("Unable to get channel listeners");
                            listeners.remove(id);

                            if listeners.is_empty() {
                                channels.remove(&name);
                                *encoded_channels = Self::encode_channels(channels);
                            }
                        }
                        ListenerType::_Group(name) => {
                            // Remove `name` from ChannelMap and re-encode
                            debug!("Removing channel group from SubscribeLoop: {}", name);

                            let listeners = groups
                                .get_mut(&name)
                                .expect("Unable to get channel listeners");
                            listeners.remove(id);

                            if listeners.is_empty() {
                                groups.remove(&name);
                                *encoded_groups = Self::encode_channels(groups);
                            }
                        }
                    }

                    return channels.is_empty() && groups.is_empty();
                }
                PipeMessage::Add(listener, mut channel_tx) => {
                    let id = match listener {
                        ListenerType::Channel(name) => {
                            // Add `name` to channel and re-encode
                            debug!("Adding channel to SubscribeLoop: {}", name);

                            let listeners = channels.entry(name).or_insert_with(Default::default);
                            let id = listeners.counter();
                            listeners.push(channel_tx.clone());

                            *encoded_channels = Self::encode_channels(channels);

                            id
                        }
                        ListenerType::_Group(name) => {
                            // Add `name` to group and re-encode
                            debug!("Adding channel group to SubscribeLoop: {}", name);

                            let listeners = groups.entry(name).or_insert_with(Default::default);
                            let id = listeners.counter();
                            listeners.push(channel_tx.clone());

                            *encoded_groups = Self::encode_channels(groups);

                            id
                        }
                    };

                    // Send Subscription id
                    let msg = Message {
                        message_type: Type::Ready(id),
                        ..Message::default()
                    };
                    channel_tx
                        .send(msg)
                        .await
                        .expect("Unable to send subscription id");

                    return false;
                }
                _ => (),
            }
        }

        true
    }

    /// # Encode the channel list to a string
    ///
    /// This is also used for encoding the list of channel groups.
    fn encode_channels(channels: &ChannelMap) -> String {
        channels
            .keys()
            .map(|channel| utf8_percent_encode(channel, NON_ALPHANUMERIC).to_string())
            .collect::<Vec<_>>()
            .as_slice()
            .join("%2C")
    }
}
