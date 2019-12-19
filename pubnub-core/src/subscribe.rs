use std::pin::Pin;

use crate::message::{Message, Timetoken, Type};
use crate::pipe::{ListenerType, Pipe, PipeMessage, PipeTx};
use crate::runtime::Runtime;
use crate::transport::Transport;
use futures_util::future::FutureExt;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::{Context, Poll};
use log::{debug, error};

pub(crate) mod channel;
// mod control;
mod encoded_channels_list;
mod registry;
mod subscribe_request;

pub(crate) use channel::{Rx as ChannelRx, Tx as ChannelTx};
use encoded_channels_list::EncodedChannelsList;
use registry::{RegistrationEffect, Registry as GenericRegistry, UnregistrationEffect};

pub use registry::ID as SubscriptionID;

pub(crate) type Registry = GenericRegistry<ChannelTx>;

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

#[derive(Debug)]
pub(crate) struct SubscribeLoopParams<TTransport> {
    pub control_pipe: Pipe,

    pub transport: TTransport,

    pub origin: String,
    pub agent: String,
    pub subscribe_key: String,

    pub channels: Registry,
    pub groups: Registry,
}

/// Implements the subscribe loop, which efficiently polls for new messages.
pub(crate) async fn subscribe_loop<TTransport>(params: SubscribeLoopParams<TTransport>)
where
    TTransport: Transport,
    <TTransport as Transport>::Error: 'static,
{
    debug!("Starting subscribe loop");

    #[allow(clippy::unneeded_field_pattern)]
    let SubscribeLoopParams {
        mut control_pipe,

        transport,

        origin,
        agent: _, // TODO: use it.
        subscribe_key,

        mut channels,
        mut groups,
    } = params;

    let mut encoded_channels = EncodedChannelsList::from(&channels);
    let mut encoded_groups = EncodedChannelsList::from(&groups);

    let mut timetoken = Timetoken::default();
    let mut control_pipe_rx = control_pipe.rx.fuse();

    loop {
        // TODO: do not clone cached values cause it defeats the purpose of the
        // cache. Do we need this cache though? There no benchmarks...
        let request_encoded_channels = encoded_channels.clone();
        let response = subscribe_request::subscribe_request(
            &transport,
            subscribe_request::SubscribeRequestParams {
                origin: &origin,
                subscribe_key: &subscribe_key,
                encoded_channels: &request_encoded_channels,
            },
            timetoken.clone(),
        )
        .fuse();
        futures_util::pin_mut!(response);

        #[allow(clippy::mut_mut)]
        let (messages, next_timetoken) = futures_util::select! {
            // This is ugly, but necessary. Moving these into a sub-struct might clean it up.
            // See: http://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/
            msg = control_pipe_rx.next() => {
                let outcome = handle_control_command(
                    &mut channels,
                    &mut groups,
                    &mut encoded_channels,
                    &mut encoded_groups,
                    msg,
                ).await;
                if let ControlOutcome::Terminate = outcome {
                    // Termination requested, break the loop.
                    break;
                }

                // Control signalled we can continue with the polling, however
                // we literally need to `continue` here in order to force rerun
                // the loop from the beginning.
                // We rely on the in-flight request to be properly cleaned up,
                // since their futures are being dropped here.
                continue;
            },

            res = response => {
                match res {
                    Ok(v) => v,
                    Err(err) => {
                        // TODO: add some kind of circut breaker.
                        // Report error and retry - maybe it'd work this time.
                        error!("Transport error while polling: {:?}", err);
                        continue;
                    }
                }
            }
        };

        // Send ready message when the subscribe loop is capable of receiving
        // messages.
        // This is intended to signal the readiness (and the healthiness) of
        // the setup. It is invoked after the `Ok` result from the request
        // future, guaranteing that Transport was able to perform successfully
        // at least once.
        // TODO: decouple signalling from the control channel into a separate
        // oneshot channel. It will simplify the `PipeMessage` type and logic
        // around it, and will allow for opting-in to this signal if needed.
        if timetoken.t == 0 {
            if let Err(error) = control_pipe.tx.send(PipeMessage::Ready).await {
                error!("Error sending ready message: {:?}", error);
                break;
            }
        }

        // Save Timetoken for next request
        timetoken = next_timetoken;

        debug!("messages: {:?}", messages);
        debug!("timetoken: {:?}", timetoken);

        // Distribute messages to each listener.
        for message in messages {
            let route = message
                .route
                .clone()
                .unwrap_or_else(|| message.channel.clone());
            debug!("route: {}", route);

            // TODO: provide a better interface and remove the potentially
            // unsound `get` and `get_mut` from the registry API.
            let listeners = channels.get_mut(&route).unwrap();

            debug!("Delivering to {} listeners...", listeners.len());
            for channel_tx in listeners.iter_mut() {
                if let Err(error) = channel_tx.send(message.clone()).await {
                    error!("Delivery error: {:?}", error);
                }
            }
        }
    }

    debug!("Stopping subscribe loop");

    // TODO: only send exit message when testing.
    // TODO: use oneshot channel for this signal and allow opting in to it
    // if user needs it at compile time (via generics rather than via features).
    // We started sending this plainly to ease testing after we
    // split the crates.
    // #[cfg(test)]
    control_pipe
        .tx
        .send(PipeMessage::Exit)
        .await
        .expect("Unable to send exit message");
}

/// Encodes action to be taken in response to control command.
#[derive(Debug)]
enum ControlOutcome {
    Terminate,
    CanContinue,
}

/// Handle a control command
///
/// This is split out from the `select!` macro used in the `SubscribeLoop`. Debugging complex
/// code buried within a macro is very painful. So this allows our development experience to be
/// flexible and the compiler can actually show us useful errors.
async fn handle_control_command(
    channels: &mut Registry,
    groups: &mut Registry,
    encoded_channels: &mut EncodedChannelsList,
    encoded_groups: &mut EncodedChannelsList,
    msg: Option<PipeMessage>,
) -> ControlOutcome {
    debug!("Got request: {:?}", msg);
    let request = match msg {
        Some(v) => v,
        None => return ControlOutcome::CanContinue,
    };
    match request {
        PipeMessage::Drop(id, listener) => {
            // Remove channel or group listener.
            let (name, kind, registry, cache, other_is_empty) = match listener {
                ListenerType::Channel(name) => (
                    name,
                    "channel",
                    channels,
                    encoded_channels,
                    groups.is_empty(),
                ),
                ListenerType::_Group(name) => {
                    (name, "group", groups, encoded_groups, channels.is_empty())
                }
            };

            // Unregister the listener from the registry.
            debug!("Removing {} from SubscribeLoop: {}", kind, name);

            let (_, effect) = registry
                .unregister(&name, id)
                .expect("Unable to get channel listeners");

            // Update cache if needed.
            if let UnregistrationEffect::NameErazed = effect {
                *cache = EncodedChannelsList::from(&*registry);
            }

            // TODO: avoid terminating loop here avoid special casing.
            if other_is_empty && registry.is_empty() {
                ControlOutcome::Terminate
            } else {
                ControlOutcome::CanContinue
            }
        }
        PipeMessage::Add(listener, mut channel_tx) => {
            // Add channel or group listener.
            let (name, kind, registry, cache) = match listener {
                ListenerType::Channel(name) => (name, "channel", channels, encoded_channels),
                ListenerType::_Group(name) => (name, "group", groups, encoded_groups),
            };
            debug!("Adding {} to SubscribeLoop: {}", kind, name);

            // Register new channel listener with the registry.
            let (id, effect) = registry.register(name, channel_tx.clone());

            // Update cache if needed.
            if let RegistrationEffect::NewName = effect {
                *cache = EncodedChannelsList::from(&*registry);
            }

            // Send Subscription ID.
            // TODO: avoid mixing signalling and data "planes" (channels).
            // Use oneshot channel to communicate the ID back to the
            // `Subscription` struct.
            let msg = Message {
                message_type: Type::Ready(id),
                ..Message::default()
            };
            channel_tx
                .send(msg)
                .await
                .expect("Unable to send subscription id");

            ControlOutcome::CanContinue
        }
        _ => ControlOutcome::CanContinue,
    }
}
