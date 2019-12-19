use crate::message::{Message, Timetoken, Type};
use crate::pipe::{ListenerType, Pipe, PipeMessage, SharedPipe};
use crate::runtime::Runtime;
use crate::subscribe::Registry;
use crate::subscribe::{subscribe_loop, SubscribeLoopParams, Subscription};
use crate::transport::Transport;
use futures_util::stream::StreamExt;
use json::JsonValue;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tokio::sync::mpsc;

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

    pub(crate) origin: String, // "domain:port"
    // TODO: unexpose.
    pub agent: String, // "Rust-Agent"
    // TODO: unexpose.
    pub publish_key: String, // Customer's Publish Key
    // TODO: unexpose.
    pub subscribe_key: String,             // Customer's Subscribe Key
    pub(crate) secret_key: Option<String>, // Customer's Secret Key
    pub(crate) auth_key: Option<String>,   // Client Auth Key for R+W Access
    pub(crate) user_id: Option<String>,    // Client UserId "UUID" for Presence
    pub(crate) filters: Option<String>,    // Metadata Filters on Messages
    pub(crate) presence: bool,             // Enable presence events

    // TODO: unexpose.
    pub pipe: SharedPipe, // Allows communication with a subscribe loop
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
        let (channel_tx, mut channel_rx) = mpsc::channel(10);

        // TODO: refactor this logic.

        // Hold the lock for the entire duration of this function
        let mut control_pipe_guard = self.pipe.lock().await;

        let mut id = if let Some(control_pipe) = control_pipe_guard.as_mut() {
            // Send an "add channel" message to the subscribe loop
            let channel = ListenerType::Channel(channel.to_string());
            debug!("Adding channel: {:?}", channel);

            let result = control_pipe
                .tx
                .send(PipeMessage::Add(channel, channel_tx.clone()))
                .await;

            if result.is_ok() {
                // Fetch id from `SubscribeLoop`
                // Uses `channel_rx` which is unique to each `Subscription`.
                let msg = channel_rx.next().await;

                if let Some(Message {
                    message_type: Type::Ready(id),
                    ..
                }) = msg
                {
                    Some(id)
                } else {
                    panic!("Unexpected message: {:?}", msg);
                }
            } else {
                // FIXME: this doesn't control the actual loop, so this code
                // would simply leak the resources and misbehave; fix this by
                // implementing proper ownership model for the loop future and
                // get rid of this guard revocation logic.

                // When sending to the pipe fails, recreate the SubscribeLoop
                *control_pipe_guard = None;

                None
            }
        } else {
            None
        };

        if control_pipe_guard.is_none() {
            // Create communication pipe
            let (my_pipe, their_pipe) = {
                let (my_tx, their_rx) = mpsc::channel(1);
                let (their_tx, my_rx) = mpsc::channel(10);

                let my_pipe = Pipe {
                    tx: my_tx,
                    rx: my_rx,
                };
                let their_pipe = Pipe {
                    tx: their_tx,
                    rx: their_rx,
                };

                (my_pipe, their_pipe)
            };
            *control_pipe_guard = Some(my_pipe);

            let mut channels = Registry::new();
            let (initial_id, _) = channels.register(channel.to_string(), channel_tx);

            // Assign the ID at the outer scope.
            id = Some(initial_id);

            // Create subscribe loop
            debug!("Creating the subscribe loop");
            let subscribe_loop_params = SubscribeLoopParams {
                control_pipe: their_pipe,

                transport: self.transport.clone(),

                origin: self.origin.clone(),
                agent: self.agent.clone(),
                subscribe_key: self.subscribe_key.clone(),

                channels,
                groups: Registry::new(),
            };

            // Spawn the subscribe loop onto the runtime
            self.runtime.spawn(subscribe_loop(subscribe_loop_params));

            // Code is waiting for ready signal here. It will deadlock if the
            // signal is never received.
            // TODO: discuss and reevaluate the requirements and remove it if
            // needed, or switch to a separate oneshot channel or other method
            // of ensureing readiness (a custom Future or raw Poll+Waker thing).
            debug!("Waiting for long-poll...");
            control_pipe_guard
                .as_mut()
                .unwrap()
                .rx
                .next()
                .await
                .expect("Unable to receive ready message");
        }

        Subscription {
            runtime: self.runtime.clone(),
            name: ListenerType::Channel(channel.to_string()),
            id: id.unwrap(),
            tx: control_pipe_guard.as_ref().unwrap().tx.clone(),
            channel: channel_rx,
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
            pipe: SharedPipe::default(),
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
