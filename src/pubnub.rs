use crate::adapters::runtime::default as default_runtime;
use crate::adapters::transport::default as default_transport;
use crate::channel::ChannelMap;
use crate::message::{Message, Timetoken, Type};
use crate::pipe::{ListenerType, Pipe, PipeMessage, SharedPipe};
use crate::runtime::Runtime;
use crate::subscribe::{SubscribeLoop, Subscription};
use crate::transport::Transport;
use futures_util::stream::StreamExt;
use json::JsonValue;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use tokio::sync::mpsc;

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
    pub(crate) presence: bool,             // Enable presence events

    pub(crate) pipe: SharedPipe, // Allows communication with a subscribe loop
}

/// # PubNub Client Builder
///
/// Create a `PubNub` client using the builder pattern. Optional items can be overridden using
/// this.
#[derive(Clone, Debug)]
pub struct PubNubBuilder<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
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
    /// use pubnub::{json::object, StandardPubNub as PubNub};
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
    /// use pubnub::{json::object, PubNub};
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
    /// use pubnub::PubNub;
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

        // Hold the lock for the entire duration of this function
        let mut guard = self.pipe.lock().await;

        let id = if let Some(pipe) = guard.as_mut() {
            // Send an "add channel" message to the subscribe loop
            let channel = ListenerType::Channel(channel.to_string());
            debug!("Adding channel: {:?}", channel);

            let result = pipe
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
                    id
                } else {
                    panic!("Unexpected message: {:?}", msg);
                }
            } else {
                // When sending to the pipe fails, recreate the SubscribeLoop
                *guard = None;

                0
            }
        } else {
            0
        };

        if guard.is_none() {
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
            *guard = Some(my_pipe);

            let mut channels: ChannelMap = ChannelMap::new();
            let listeners = channels
                .entry(channel.to_string())
                .or_insert_with(Default::default);
            listeners.push(channel_tx);

            // Create subscribe loop
            debug!("Creating SubscribeLoop");
            let subscribe_loop = SubscribeLoop::new(
                their_pipe,
                self.transport.clone(),
                self.runtime.clone(),
                self.origin.clone(),
                self.agent.clone(),
                self.subscribe_key.clone(),
                channels,
                ChannelMap::new(),
            );

            // Spawn the subscribe loop onto the runtime
            self.runtime.spawn(subscribe_loop.run());

            debug!("Waiting for long-poll...");
            guard
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
            id,
            tx: guard.as_ref().unwrap().tx.clone(),
            channel: channel_rx,
        }
    }

    /// Set the subscribe filters.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub::PubNub;
    ///
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// pubnub.filters("uuid != JoeBob");
    /// ```
    pub fn filters(&mut self, filters: &str) {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
    }
}

impl PubNub<default_transport::Transport, default_runtime::Runtime> {
    /// Create a new `PubNub` client with default configuration.
    ///
    /// To create a `PubNub` client with custom configuration, use [`PubNubBuilder::new`].
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub::PubNub;
    ///
    /// let pubnub = PubNub::new("demo", "demo");
    /// ```
    #[must_use]
    pub fn new(publish_key: &str, subscribe_key: &str) -> Self {
        PubNubBuilder::new(publish_key, subscribe_key).build()
    }
}

impl PubNubBuilder<default_transport::Transport, default_runtime::Runtime> {
    /// Create a new `PubNubBuilder` that can configure a `PubNub` client.
    #[must_use]
    pub fn new(publish_key: &str, subscribe_key: &str) -> Self {
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

            transport: default_transport::Transport::default(),
            runtime: default_runtime::Runtime::default(),
        }
    }
}

#[allow(clippy::use_self)] // false positives
impl<TTransport, TRuntime> PubNubBuilder<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    /// Set the PubNub network origin.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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
    /// use pubnub::PubNubBuilder;
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

    /// Build the PubNub client to begin streaming messages.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub::PubNubBuilder;
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
