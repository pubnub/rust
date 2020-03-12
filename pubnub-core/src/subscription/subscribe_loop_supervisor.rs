use super::registry::Registry;
use super::subscribe_loop::{
    subscribe_loop, ControlCommand, ControlTx, ExitTx, SubscribeLoopParams,
};
use super::subscription::Subscription;
use crate::data::pubsub;
use crate::runtime::Runtime;
use crate::transport::Transport;
use crate::PubNub;
use futures_channel::{mpsc, oneshot};
use futures_util::sink::SinkExt;
use log::debug;

/// SubscribeLoopSupervisor is responsible for the lifecycle of the subscribe
/// loop.
/// It owns the subscribe loop control handle and provides a high-level
/// interface to the control operations.
/// It will intelligently spawn and respawn the subscribe loop on a need basis.
///
/// Deliberately doesn't implement `Clone` to avoid issues with improper
/// duplication of control handles.
#[derive(Debug)]
pub(crate) struct SubscribeLoopSupervisor {
    /// Configuration params.
    params: SubscribeLoopSupervisorParams,

    /// Control handle to the subscribe loop.
    control_tx: Option<ControlTx>,
}

/// SubscribeLoopSupervisorParams configuration params.
#[derive(Debug)]
pub(crate) struct SubscribeLoopSupervisorParams {
    /// If set, gets a signal when subscribe loop exits.
    pub exit_tx: Option<ExitTx>,
}

impl SubscribeLoopSupervisor {
    pub fn new(params: SubscribeLoopSupervisorParams) -> Self {
        Self {
            params,
            control_tx: None,
        }
    }
}

impl SubscribeLoopSupervisor {
    pub async fn subscribe<'a, TTransport, TRuntime>(
        &mut self,
        pubnub: &'a mut PubNub<TTransport, TRuntime>,
        to: pubsub::SubscribeTo,
    ) -> Subscription<TRuntime>
    where
        TTransport: Transport + 'static,
        TRuntime: Runtime + 'static,
    {
        // Since recursion is troublesome with async fns, we use the loop trick.
        let (id, control_tx, channel_rx) = loop {
            let (channel_tx, channel_rx) = mpsc::channel(10);

            let id_or_retry = if let Some(ref mut control_tx) = self.control_tx {
                // Send a command to add the channel to the running
                // subscribe loop.

                debug!("Adding destination {:?} to the running loop", to);

                let (id_tx, id_rx) = oneshot::channel();

                let control_comm_result = control_tx
                    .send(ControlCommand::Add(to.clone(), channel_tx, id_tx))
                    .await;

                if control_comm_result.is_err() {
                    // We got send error, this only happens when the receive
                    // half of the channel is closed.
                    // Assuming it was dropped because of being out of
                    // scope, we conclude the subscribe loop has completed.
                    // We simply cleanup the control tx, and retry
                    // subscribing.
                    // The successive subscribtion attempt will result in
                    // starting off of a new subscription loop and properly
                    // registering the channel there.
                    self.control_tx = None;

                    debug!("Restarting the subscription loop");

                    // This is equivalent to calling the `subscribe` fn
                    // recursively, given we're in the loop context.
                    None
                } else {
                    // We succesfully submitted the command, wait for
                    // subscription loop to communicate the subscription ID
                    // back to us.
                    let id = id_rx.await.unwrap();

                    // Return the values from the loop.
                    Some((id, control_tx.clone()))
                }
            } else {
                // Since there's no subscribe loop loop found, spawn a new
                // one.

                let mut registry = Registry::new();
                let (id, _) = registry.register(to.clone(), channel_tx);

                let (control_tx, control_rx) = mpsc::channel(10);
                let (ready_tx, ready_rx) = oneshot::channel();

                debug!("Creating the subscribe loop");
                let subscribe_loop_params = SubscribeLoopParams {
                    control_rx,
                    ready_tx: Some(ready_tx),
                    exit_tx: self.params.exit_tx.clone(),

                    transport: pubnub.transport.clone(),

                    to: registry,
                };

                // Spawn the subscribe loop onto the runtime
                pubnub.runtime.spawn(subscribe_loop(subscribe_loop_params));

                // Waiting for subscription loop to communicate that it's
                // ready.
                // Will deadlock if the signal is never received, which will
                // only happen if the subscription loop is stuck somehow.
                // If subscription loop fails and goes out of scope we'll
                // get an error properly communicating that.
                debug!("Waiting for subscription loop ready...");
                ready_rx.await.expect("Unable to receive ready message");

                // Keep the control tx for later.
                self.control_tx = Some(control_tx.clone());

                // Return the values from the loop.
                Some((id, control_tx))
            };

            match id_or_retry {
                Some((id, control_tx)) => break (id, control_tx, channel_rx),
                None => continue,
            }
        };

        Subscription {
            runtime: pubnub.runtime.clone(),
            destination: to,
            id,
            control_tx,
            channel_rx,
        }
    }
}
