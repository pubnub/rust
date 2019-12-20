use crate::message::Timetoken;
use crate::runtime::Runtime;
use crate::subscribe::{
    subscribe_loop, ControlCommand, ControlTx as SubscribeLoopControlTx,
    ExitTx as SubscribeLoopExitTx, ListenerType, Registry, SubscribeLoopParams, Subscription,
};
use crate::transport::Transport;
use json::JsonValue;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

#[cfg(test)]
mod tests;

/// # PubNub Client
///
/// The PubNub lib implements socket pools to relay data requests as a client connection to the
/// PubNub Network.
#[derive(Clone, Debug)]
pub struct PubNub<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    transport: TTransport, // Transport to use for communication
    runtime: TRuntime,     // Runtime to use for managing resources

    pub(crate) origin: String,             // "domain:port"
    pub(crate) agent: String,              // "Rust-Agent"
    pub(crate) publish_key: String,        // Customer's Publish Key
    pub(crate) subscribe_key: String,      // Customer's Subscribe Key
    pub(crate) secret_key: Option<String>, // Customer's Secret Key
    pub(crate) auth_key: Option<String>,   // Client Auth Key for R+W Access
    pub(crate) user_id: Option<String>,    // Client UserId "UUID" for Presence
    pub(crate) filters: Option<String>,    // Metadata Filters on Messages
    pub(crate) presence: bool,             // Enable presence even

    // Subscription related configuration params.
    pub(crate) subscribe_loop_exit_tx: Option<SubscribeLoopExitTx>, // If set, gets a signal when subscribe loop exits.

    // Subscription related runtime state.
    pub(crate) subscribe_loop_control_tx: Arc<Mutex<Option<SubscribeLoopControlTx>>>, // Control interface into the subscribe loop.
}

/// # PubNub Client Builder
///
/// Create a `PubNub` client using the builder pattern. Optional items can be overridden using
/// this.
#[derive(Clone, Debug)]
pub struct PubNubBuilder<TTransport, TRuntime> {
    transport: TTransport, // Transport to use for communication
    runtime: TRuntime,     // Runtime to use for managing resources

    origin: String,             // "domain:port"
    agent: String,              // "Rust-Agent"
    publish_key: String,        // Customer's Publish Key
    subscribe_key: String,      // Customer's Subscribe Key
    secret_key: Option<String>, // Customer's Secret Key
    auth_key: Option<String>,   // Client Auth Key for R+W Access
    user_id: Option<String>,    // Client UserId "UUID" for Presence
    filters: Option<String>,    // Metadata Filters on Messages
    presence: bool,             // Enable presence events

    // Subscription related configuration params.
    subscribe_loop_exit_tx: Option<SubscribeLoopExitTx>, // If set, gets a signal when subscribe loop exits.
}

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Publish a message over the PubNub network.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::{core::json::object, PubNub};
    ///
    /// # async {
    /// let pubnub = PubNub::new("demo", "demo");
    ///
    /// let timetoken = pubnub.publish("my-channel", object!{
    ///     "username" => "JoeBob",
    ///     "content" => "Hello, world!",
    /// }).await?;
    ///
    /// println!("Timetoken: {}", timetoken);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub async fn publish(
        &self,
        channel: &str,
        message: JsonValue,
    ) -> Result<Timetoken, TTransport::Error> {
        self.publish_with_metadata(channel, message, JsonValue::Null)
            .await
    }

    /// Publish a message over the PubNub network with an extra metadata payload.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::{core::json::object, PubNub};
    ///
    /// # async {
    /// let pubnub = PubNub::new("demo", "demo");
    ///
    /// let message = object!{
    ///     "username" => "JoeBob",
    ///     "content" => "Hello, world!",
    /// };
    /// let metadata = object!{
    ///     "uuid" => "JoeBob",
    /// };
    ///
    /// let timetoken = pubnub.publish_with_metadata("my-channel", message, metadata).await?;
    ///
    /// println!("Timetoken: {}", timetoken);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub async fn publish_with_metadata(
        &self,
        channel: &str,
        message: JsonValue,
        _metadata: JsonValue,
    ) -> Result<Timetoken, TTransport::Error> {
        let message = json::stringify(message);
        let message = utf8_percent_encode(&message, NON_ALPHANUMERIC);
        let channel = utf8_percent_encode(channel, NON_ALPHANUMERIC);

        // Construct URI
        // TODO:
        // - auth key
        // - uuid
        // - signature
        let url = format!(
            "https://{origin}/publish/{pub_key}/{sub_key}/0/{channel}/0/{message}",
            origin = self.origin,
            pub_key = self.publish_key,
            sub_key = self.subscribe_key,
            channel = channel,
            message = message,
        );
        debug!("URL: {}", url);

        // Send network request
        let url = url.parse().expect("Unable to parse URL");
        self.transport.publish_request(url).await
    }

    /// Subscribe to a message stream over the PubNub network.
    ///
    /// The PubNub client only maintains a single subscribe loop for all subscription streams. This
    /// has a benefit that it optimizes for a low number of sockets to the PubNub network. It has a
    /// downside that requires _all_ streams to consume faster than the subscribe loop produces.
    /// A slow consumer will create a head-of-line blocking bottleneck in the processing of
    /// received messages. All streams can only consume as fast as the slowest.
    ///
    /// For example, with 3 total subscription streams and 1 that takes 30 seconds to process each
    /// message; the other 2 streams will be blocked waiting for that 30-second duration on the
    /// slow consumer.
    ///
    /// To workaround this problem, you may consider enabling reduced resiliency with
    /// [`PubNubBuilder::reduced_resliency`], which will drop messages on the slowest consumers,
    /// allowing faster consumers to continue processing messages without blocking.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures_util::stream::StreamExt;
    /// use pubnub_hyper::PubNub;
    ///
    /// # async {
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// let mut stream = pubnub.subscribe("my-channel").await;
    ///
    /// while let Some(message) = stream.next().await {
    ///     println!("Received message: {:?}", message);
    /// }
    /// # };
    /// ```
    pub async fn subscribe(&mut self, channel: &str) -> Subscription<TRuntime> {
        // Since recursion is troublesome with async fns, we use the loop trick.
        let (id, control_tx, channel_rx) = loop {
            let (channel_tx, channel_rx) = mpsc::channel(10);

            let mut subscribe_loop_control_tx_guard = self.subscribe_loop_control_tx.lock().await;

            let id_or_retry = if let Some(ref mut control_tx) = *subscribe_loop_control_tx_guard {
                // Send a command to add the channel to the running
                // subscribe loop.

                debug!("Adding channel {:?} to the running loop", channel);

                let (id_tx, id_rx) = oneshot::channel();

                // TODO: unify interfaces to either use `ListenerType` or
                // `&str` when we refer to a channel.
                let listener = ListenerType::Channel(channel.to_string());

                let control_comm_result = control_tx
                    .send(ControlCommand::Add(listener, channel_tx, id_tx))
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
                    *subscribe_loop_control_tx_guard = None;

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

                let mut channels = Registry::new();
                let (id, _) = channels.register(channel.to_string(), channel_tx);

                let (control_tx, control_rx) = mpsc::channel(10);
                let (ready_tx, ready_rx) = oneshot::channel();

                debug!("Creating the subscribe loop");
                let subscribe_loop_params = SubscribeLoopParams {
                    control_rx,
                    ready_tx: Some(ready_tx),
                    exit_tx: self.subscribe_loop_exit_tx.clone(),

                    transport: self.transport.clone(),

                    origin: self.origin.clone(),
                    agent: self.agent.clone(),
                    subscribe_key: self.subscribe_key.clone(),

                    channels,
                    groups: Registry::new(),
                };

                // Spawn the subscribe loop onto the runtime
                self.runtime.spawn(subscribe_loop(subscribe_loop_params));

                // Waiting for subscription loop to communicate that it's
                // ready.
                // Will deadlock if the signal is never received, which will
                // only happen if the subscription loop is stuck somehow.
                // If subscription loop fails and goes out of scope we'll
                // get an error properly communicating that.
                debug!("Waiting for subscription loop ready...");
                ready_rx.await.expect("Unable to receive ready message");

                // Keep the control tx for later.
                *subscribe_loop_control_tx_guard = Some(control_tx.clone());

                // Return the values from the loop.
                Some((id, control_tx))
            };

            match id_or_retry {
                Some((id, control_tx)) => break (id, control_tx, channel_rx),
                None => continue,
            }
        };

        Subscription {
            runtime: self.runtime.clone(),
            name: ListenerType::Channel(channel.to_string()),
            id,
            control_tx,
            channel_rx,
        }
    }

    /// Set the subscribe filters.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNub;
    ///
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// pubnub.filters("uuid != JoeBob");
    /// ```
    pub fn filters(&mut self, filters: &str) {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
    }

    /// Get a reference to a transport being used.
    pub fn transport(&self) -> &TTransport {
        &self.transport
    }

    /// Get a reference to a runtime being used.
    pub fn runtime(&self) -> &TRuntime {
        &self.runtime
    }
}

impl<TTransport, TRuntime> PubNubBuilder<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    /// Build the PubNub client to begin streaming messages.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> PubNub<TTransport, TRuntime> {
        PubNub {
            transport: self.transport,
            runtime: self.runtime,
            origin: self.origin,
            agent: self.agent,
            publish_key: self.publish_key,
            subscribe_key: self.subscribe_key,
            secret_key: self.secret_key,
            auth_key: self.auth_key,
            user_id: self.user_id,
            filters: self.filters,
            presence: self.presence,

            subscribe_loop_exit_tx: self.subscribe_loop_exit_tx,

            subscribe_loop_control_tx: Arc::new(Mutex::new(None)),
        }
    }
}

#[allow(clippy::use_self)] // false positives
impl<TTransport, TRuntime> PubNubBuilder<TTransport, TRuntime> {
    /// Create a new `PubNubBuilder` that can configure a `PubNub` client
    /// with custom components implementations.
    #[must_use]
    pub fn with_components(
        publish_key: &str,
        subscribe_key: &str,
        transport: TTransport,
        runtime: TRuntime,
    ) -> Self {
        Self {
            origin: "ps.pndsn.com".to_string(),
            agent: "Rust-Agent".to_string(),
            publish_key: publish_key.to_string(),
            subscribe_key: subscribe_key.to_string(),
            secret_key: None,
            auth_key: None,
            user_id: None,
            filters: None,
            presence: false,
            subscribe_loop_exit_tx: None,

            transport,
            runtime,
        }
    }

    /// Set the PubNub network origin.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .origin("pubsub.pubnub.com")
    ///     .build();
    /// ```
    #[must_use]
    pub fn origin(mut self, origin: &str) -> Self {
        self.origin = origin.to_string();
        self
    }

    /// Set the HTTP user agent string.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .agent("My Awesome Rust App/1.0.0")
    ///     .build();
    /// ```
    #[must_use]
    pub fn agent(mut self, agent: &str) -> Self {
        self.agent = agent.to_string();
        self
    }

    /// Set the PubNub secret key.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .secret_key("sub-c-deadbeef-0000-1234-abcd-c0deface")
    ///     .build();
    /// ```
    #[must_use]
    pub fn secret_key(mut self, secret_key: &str) -> Self {
        self.secret_key = Some(secret_key.to_string());
        self
    }

    /// Set the PubNub PAM auth key.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .auth_key("Open-Sesame!")
    ///     .build();
    /// ```
    #[must_use]
    pub fn auth_key(mut self, auth_key: &str) -> Self {
        self.auth_key = Some(auth_key.to_string());
        self
    }

    /// Set the PubNub User ID (Presence UUID).
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .user_id("JoeBob")
    ///     .build();
    /// ```
    #[must_use]
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Set the subscribe filters.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .filters("uuid != JoeBob")
    ///     .build();
    /// ```
    #[must_use]
    pub fn filters(mut self, filters: &str) -> Self {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
        self
    }

    /// Enable or disable interest in receiving Presence events.
    ///
    /// When enabled (default), `pubnub.subscribe()` will provide messages with type
    /// `MessageType::Presence` when users join and leave the channels you are listening on.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .presence(true)
    ///     .build();
    /// ```
    #[must_use]
    pub fn presence(mut self, enable: bool) -> Self {
        self.presence = enable;
        self
    }

    /// Enable or disable dropping messages on slow streams.
    ///
    /// When disabled (default), `pubnub.subscribe()` will provide _all_ messages to _all_ streams,
    /// regardless of how long each stream consumer takes. This provides high resilience (minimal
    /// message loss) at the cost of higher latency for streams that are blocked waiting for the
    /// slowest stream.
    ///
    /// See: [Head-of-line blocking](https://en.wikipedia.org/wiki/Head-of-line_blocking).
    ///
    /// When enabled, the subscription will drop messages to the slowest streams, improving latency
    /// for all other streams.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .reduced_resliency(true)
    ///     .build();
    /// ```
    #[must_use]
    pub fn reduced_resliency(self, _enable: bool) -> Self {
        // TODO:
        let _ = self;
        unimplemented!("Reduced resiliency is not yet available");
    }

    /// Set the subscribe loop exit tx.
    ///
    /// If set, subscribe loop sends a message to it when it exits.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pubnub_hyper::PubNubBuilder;
    ///
    /// let (tx, _rx) = tokio::sync::mpsc::channel(1);
    ///
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .subscribe_loop_exit_tx(tx)
    ///     .build();
    /// ```
    #[must_use]
    pub fn subscribe_loop_exit_tx(mut self, tx: SubscribeLoopExitTx) -> Self {
        self.subscribe_loop_exit_tx = Some(tx);
        self
    }

    /// Transport.
    ///
    /// A transport implementation to use.
    #[must_use]
    pub fn transport<U: Transport>(self, transport: U) -> PubNubBuilder<U, TRuntime> {
        PubNubBuilder {
            transport,

            // Copy the rest of the fields
            origin: self.origin,
            agent: self.agent,
            publish_key: self.publish_key,
            subscribe_key: self.subscribe_key,
            secret_key: self.secret_key,
            auth_key: self.auth_key,
            user_id: self.user_id,
            filters: self.filters,
            presence: self.presence,
            subscribe_loop_exit_tx: self.subscribe_loop_exit_tx,

            runtime: self.runtime,
        }
    }

    /// Runtime.
    ///
    /// A runtime implementation to use.
    #[must_use]
    pub fn runtime<U: Runtime>(self, runtime: U) -> PubNubBuilder<TTransport, U> {
        PubNubBuilder {
            runtime,

            // Copy the rest of the fields
            origin: self.origin,
            agent: self.agent,
            publish_key: self.publish_key,
            subscribe_key: self.subscribe_key,
            secret_key: self.secret_key,
            auth_key: self.auth_key,
            user_id: self.user_id,
            filters: self.filters,
            presence: self.presence,
            subscribe_loop_exit_tx: self.subscribe_loop_exit_tx,

            transport: self.transport,
        }
    }
}

mod default {
    use super::*;

    impl<TTransport, TRuntime> PubNubBuilder<TTransport, TRuntime>
    where
        TTransport: Default,
        TRuntime: Default,
    {
        /// Create a new `PubNubBuilder` that can configure a `PubNub` client
        /// with default components.
        #[must_use]
        pub fn new(publish_key: &str, subscribe_key: &str) -> Self {
            Self::with_components(
                publish_key,
                subscribe_key,
                TTransport::default(),
                TRuntime::default(),
            )
        }
    }

    impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
    where
        TTransport: Transport + Default,
        TRuntime: Runtime + Default,
    {
        /// Create a new `PubNub` client with default configuration.
        ///
        /// To create a `PubNub` client with custom configuration, use [`PubNubBuilder::new`].
        ///
        /// # Example
        ///
        /// ```
        /// use pubnub_hyper::PubNub;
        ///
        /// let pubnub = PubNub::new("demo", "demo");
        /// ```
        #[must_use]
        pub fn new(publish_key: &str, subscribe_key: &str) -> Self {
            PubNubBuilder::new(publish_key, subscribe_key).build()
        }
    }
}
