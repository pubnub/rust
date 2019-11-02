#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]

//! # Async PubNub Client SDK for Rust
//!
//! - Fully `async`/`await` ready.
//! - Uses Tokio and Hyper to provide an ultra-fast, incredibly reliable message transport over the
//!   PubNub edge network.
//! - Optimizes for minimal network sockets with an infinite number of logical streams.

use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use crate::mvec::MVec;
use futures_util::future::FutureExt;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::{Context, Poll};
use hyper::{client::HttpConnector, Uri};
use hyper_tls::HttpsConnector;
pub use json::JsonValue;
use log::{debug, error};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use thiserror::Error;
use tokio::sync::mpsc;

mod mvec;

type HttpClient = hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>;
type ChannelTx = mpsc::Sender<Message>;
type ChannelRx = mpsc::Receiver<Message>;
type ChannelMap = HashMap<String, MVec<ChannelTx>>;
type PipeTx = mpsc::Sender<PipeMessage>;
type PipeRx = mpsc::Receiver<PipeMessage>;

/// # PubNub Client
///
/// The PubNub lib implements socket pools to relay data requests as a client connection to the
/// PubNub Network.
#[derive(Debug)]
pub struct PubNub {
    origin: String,             // "domain:port"
    agent: String,              // "Rust-Agent"
    client: HttpClient,         // HTTP Client
    publish_key: String,        // Customer's Publish Key
    subscribe_key: String,      // Customer's Subscribe Key
    secret_key: Option<String>, // Customer's Secret Key
    auth_key: Option<String>,   // Client Auth Key for R+W Access
    user_id: Option<String>,    // Client UserId "UUID" for Presence
    filters: Option<String>,    // Metadata Filters on Messages
    presence: bool,             // Enable presence events
    pipe: Option<Pipe>,         // Allows communication with a subscribe loop
}

/// # PubNub Client Builder
///
/// Create a `PubNub` client using the builder pattern. Optional items can be overridden using
/// this.
#[derive(Debug, Clone)]
pub struct PubNubBuilder {
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

/// # PubNub Timetoken
///
/// This is the timetoken structure that PubNub uses as a stream index. It allows clients to
/// resume streaming from where they left off for added resiliency.
#[derive(Debug, Clone)]
pub struct Timetoken {
    t: String, // Timetoken
    r: u32,    // Origin region
}

/// # PubNub Message
///
/// This is the message structure yielded by [`Subscription`].
#[derive(Debug, Clone)]
pub struct Message {
    /// Enum Type of Message
    pub message_type: MessageType,
    /// Wildcard channel or channel group
    pub route: Option<String>,
    /// Origin Channel of Message Receipt
    pub channel: String,
    /// Decoded JSON Message Payload
    pub json: JsonValue,
    /// Metadata of Message
    pub metadata: JsonValue,
    /// Message ID Timetoken
    pub timetoken: Timetoken,
    /// Issuing client ID
    pub client: Option<String>,
    /// Subscribe key associated with the message
    pub subscribe_key: String,
    /// Message flags
    pub flags: u32,
}

/// # PubNub Subscription
///
/// This is the message stream returned by `pubnub.subscribe()`. The stream yields [`Message`]
/// items until it is dropped.
#[derive(Debug)]
pub struct Subscription {
    name: ListenerType, // Channel or Group name
    id: usize,          // Unique identifier for the listener
    tx: PipeTx,         // For interrupting the existing subscribe loop when dropped
    channel: ChannelRx, // Stream that produces messages
}

/// # PubNub Subscribe Loop
///
/// Manages state for a subscribe loop. Can be restarted by creating or dropping a `Subscription`.
/// Subscribe loops will stay active until the last `Subscription` is dropped. (Similar to `Rc` or
/// `Arc`.)
#[derive(Debug)]
struct SubscribeLoop {
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

/// # Bidirectional communication pipe for `SubscribeLoop`
///
/// `PubNub` owns a reference to one end of the pipe for communiating with the `SubscribeLoop`.
/// `Subscription` owns a clone of the sending side of the pipe to the `SubscribeLoop`, but not the
/// receiving side. `SubscribeLoop` is capable of sending messages to `PubNub`, and receiving
/// messages from both `PubNub` and any number of `Subscription` streams.
///
/// See [`PipeMessage`] for more details.
#[derive(Debug)]
struct Pipe {
    tx: PipeTx, // Send-side for bidirectional communication
    rx: PipeRx, // Recv-side for bidirectional communication
}

/// # The kinds of messages allowed to be delivered over a `Pipe`
#[derive(Debug)]
enum PipeMessage {
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
enum ListenerType {
    Channel(String), // Channel name
    _Group(String),  // Channel Group name
}

/// # PubNub Message Types
///
/// PubNub delivers multiple kinds of messages. This enumeration describes the various types
/// available.
///
/// The special `Unknown` variant may be delivered as the PubNub service evolves. It allows
/// applications built on the PubNub Rust client to be forward-compatible without requiring a full
/// client upgrade.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageType {
    /// A class message containing arbitrary payload data.
    Publish,
    /// A Lightweight message.
    Signal,
    /// An Objects service event, like space description updated.
    Objects,
    /// A message action event.
    Action,
    /// Presence event from channel (e.g. another client joined).
    Presence,
    /// Unknown type. The value may have special meaning in some contexts.
    Unknown(u32),

    #[doc(hidden)]
    /// Internal message for synchronization between `Subscription` and `SubscribeLoop`.
    Ready(usize),
}

/// # Error variants
#[derive(Debug, Error)]
pub enum Error {
    /// Hyper client error.
    #[error("Hyper client error")]
    Hyper(#[from] hyper::Error),

    /// Invalid UTF-8.
    #[error("Invalid UTF-8")]
    Utf8(#[from] std::str::Utf8Error),

    /// Invalid JSON.
    #[error("Invalid JSON")]
    Json(#[from] json::Error),
}

/// # PubNub core client
///
/// This is the base structure which manages the primary subscribe loop and provides methods for
/// sending and receiving messages in real time.
///
/// ```no_run
/// # use pubnub::PubNub;
/// use json::object;
///
/// # async {
/// let pubnub = PubNub::new("demo", "demo");
/// let status = pubnub.publish("my-channel", object!{
///     "username" => "JoeBob",
///     "content" => "Hello, world!",
/// }).await;
/// # };
/// ```
impl PubNub {
    /// # Create a new `PubNub` client with default configuration
    ///
    /// To create a `PubNub` client with custom configuration, use [`PubNubBuilder::new`].
    #[must_use]
    pub fn new(publish_key: &str, subscribe_key: &str) -> Self {
        PubNubBuilder::new(publish_key, subscribe_key).build()
    }

    /// # Publish a message over the PubNub network
    ///
    /// ```no_run
    /// # use pubnub::PubNub;
    /// use json::object;
    ///
    /// # async {
    /// let pubnub = PubNub::new("demo", "demo");
    /// let status = pubnub.publish("my-channel", object!{
    ///     "username" => "JoeBob",
    ///     "content" => "Hello, world!",
    /// }).await;
    /// # };
    /// ```
    pub async fn publish(&self, channel: &str, message: JsonValue) -> Result<Timetoken, Error> {
        self.publish_with_metadata(channel, message, JsonValue::Null)
            .await
    }

    /// # Publish a message over the PubNub network with an extra metadata payload
    ///
    /// ```no_run
    /// # use pubnub::PubNub;
    /// use json::object;
    ///
    /// # async {
    /// let pubnub = PubNub::new("demo", "demo");
    /// let message = object!{
    ///     "username" => "JoeBob",
    ///     "content" => "Hello, world!",
    /// };
    /// let metadata = object!{
    ///     "uuid" => "JoeBob",
    /// };
    /// let status = pubnub.publish_with_metadata("my-channel", message, metadata).await;
    /// # };
    /// ```
    pub async fn publish_with_metadata(
        &self,
        channel: &str,
        message: JsonValue,
        _metadata: JsonValue,
    ) -> Result<Timetoken, Error> {
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

        dbg!(&url);

        // Send network request
        let url = url.parse().expect("Unable to parse URL");
        publish_request(&self.client, url).await
    }

    /// # Subscribe to a message stream over the PubNub network
    ///
    /// The PubNub client only maintains a single subscribe loop for all subscription streams. This
    /// has a benefit that it optimizes for a low number of sockets to the PubNub network. It has a
    /// downside that requires _all_ streams to consume faster than the subscribe loop produces.
    /// A slow consumer will create a head-on-line blocking bottleneck in the processing of
    /// received messages. All streams can only consume as fast as the slowest.
    ///
    /// For example, with 3 total subscription streams and 1 that takes 30 seconds to process each
    /// message; the other 2 streams will be blocked waiting for that 30-second duration on the
    /// slow consumer.
    ///
    /// To workaround this problem, you may consider enabling reduced resiliency in
    /// [`PubNubBuilder::reduced_resliency`], which will drop messages on the slowest consumers,
    /// allowing faster consumers to continueprocessing messages without blocking.
    ///
    /// ```no_run
    /// # use pubnub::PubNub;
    /// use futures_util::stream::StreamExt;
    /// # async {
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// let mut stream = pubnub.subscribe("my-channel").await;
    /// while let Some(message) = stream.next().await {
    ///     println!("Received message: {:?}", message);
    /// }
    /// # };
    /// ```
    pub async fn subscribe(&mut self, channel: &str) -> Subscription {
        let (channel_tx, mut channel_rx) = mpsc::channel(10);

        let id = if let Some(pipe) = &mut self.pipe {
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
                    message_type: MessageType::Ready(id),
                    ..
                }) = msg
                {
                    id
                } else {
                    panic!("Unexpected message: {:?}", msg);
                }
            } else {
                // When sending to the pipe fails, recreate the SubscribeLoop
                self.pipe = None;

                0
            }
        } else {
            0
        };

        if self.pipe.is_none() {
            // Create communication pipe
            let (my_pipe, their_pipe) = {
                let (my_tx, their_rx) = mpsc::channel(10);
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
            self.pipe = Some(my_pipe);

            let mut channels: ChannelMap = HashMap::new();
            let listeners = channels
                .entry(channel.to_string())
                .or_insert_with(Default::default);
            listeners.push(channel_tx);

            // Create subscribe loop
            debug!("Creating SubscribeLoop");
            let subscribe_loop = SubscribeLoop::new(
                their_pipe,
                self.client.clone(),
                self.origin.clone(),
                self.agent.clone(),
                self.subscribe_key.clone(),
                channels,
                HashMap::new(),
            );

            // Spawn the subscribe loop onto the Tokio runtime
            tokio::spawn(subscribe_loop.run());

            debug!("Waiting for long-poll...");
            self.pipe
                .as_mut()
                .unwrap()
                .rx
                .next()
                .await
                .expect("Unable to receive ready message");
        }

        Subscription {
            name: ListenerType::Channel(channel.to_string()),
            id,
            tx: self.pipe.as_ref().unwrap().tx.clone(),
            channel: channel_rx,
        }
    }

    /// # Set the subscribe filters
    ///
    /// ```no_run
    /// # use pubnub::PubNub;
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// pubnub.filters("uuid != JoeBob");
    /// ```
    pub fn filters(&mut self, filters: &str) {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
    }
}

/// # PubNub Client Builder
///
/// Create a builder that sets sane defaults and provides methods to customize the PubNub instance
/// that it will build.
///
/// ```no_run
/// # use pubnub::PubNubBuilder;
/// let pubnub = PubNubBuilder::new("demo", "demo")
///     .origin("pubsub.pubnub.com")
///     .agent("My Awesome Rust App/1.0.0")
///     .build();
/// ```
impl PubNubBuilder {
    /// # Create a new `PubNubBuilder` that can configure a `PubNub` client
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
        }
    }

    /// # Set the PubNub network origin
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .origin("pubsub.pubnub.com")
    ///     .build();
    /// ```
    #[must_use]
    pub fn origin(mut self, origin: &str) -> Self {
        self.origin = origin.to_string();
        self
    }

    /// # Set the HTTP user agent string
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .agent("My Awesome Rust App/1.0.0")
    ///     .build();
    /// ```
    #[must_use]
    pub fn agent(mut self, agent: &str) -> Self {
        self.agent = agent.to_string();
        self
    }

    /// # Set the PubNub secret key
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .secret_key("sub-c-deadbeef-0000-1234-abcd-c0deface")
    ///     .build();
    /// ```
    #[must_use]
    pub fn secret_key(mut self, secret_key: &str) -> Self {
        self.secret_key = Some(secret_key.to_string());
        self
    }

    /// # Set the PubNub PAM auth key
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .auth_key("Open-Sesame!")
    ///     .build();
    /// ```
    #[must_use]
    pub fn auth_key(mut self, auth_key: &str) -> Self {
        self.auth_key = Some(auth_key.to_string());
        self
    }

    /// # Set the PubNub User ID (Presence UUID)
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .user_id("JoeBob")
    ///     .build();
    /// ```
    #[must_use]
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// # Set the subscribe filters
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .filters("uuid != JoeBob")
    ///     .build();
    /// ```
    #[must_use]
    pub fn filters(mut self, filters: &str) -> Self {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
        self
    }

    /// # Enable or disable interest in receiving Presence events
    ///
    /// When enabled (default), `pubnub.subscribe()` will provide messages with type
    /// `MessageType::Presence` when users join and leave the channels you are listening on.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .presence(true)
    ///     .build();
    /// ```
    #[must_use]
    pub fn presence(mut self, enable: bool) -> Self {
        self.presence = enable;
        self
    }

    /// # Enable or disable dropping messages on slow streams
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
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
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

    /// # Build the PubNub client to begin streaming messages
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> PubNub {
        let https = HttpsConnector::new().unwrap();
        let client = hyper::Client::builder()
            .keep_alive_timeout(Some(Duration::from_secs(300)))
            .max_idle_per_host(10000)
            .build::<_, hyper::Body>(https);

        PubNub {
            origin: self.origin,
            agent: self.agent,
            client,
            publish_key: self.publish_key,
            subscribe_key: self.subscribe_key,
            secret_key: self.secret_key,
            auth_key: self.auth_key,
            user_id: self.user_id,
            filters: self.filters,
            presence: self.presence,
            pipe: None,
        }
    }
}

impl MessageType {
    /// # Create a `MessageType` from an integer
    ///
    /// Subscribe message pyloads include a non-enumerated integer to describe message types. We
    /// instead provide a concrete type, using this function to convert the integer into the
    /// appropriate type.
    fn from_json(i: &JsonValue) -> Self {
        match i.as_u32().unwrap_or(0) {
            0 => Self::Publish,
            1 => Self::Signal,
            2 => Self::Objects,
            3 => Self::Action,
            i => Self::Unknown(i),
        }
    }
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
    fn new(
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
    async fn run(mut self) {
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
            if timetoken.t == "0" {
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
    async fn handle_request(
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

                    if channels.is_empty() && groups.is_empty() {
                        return true;
                    }
                    return false;
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
                        message_type: MessageType::Ready(id),
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

impl Default for Message {
    #[must_use]
    fn default() -> Self {
        Self {
            message_type: MessageType::Unknown(0),
            route: Option::default(),
            channel: String::default(),
            json: JsonValue::Null,
            metadata: JsonValue::Null,
            timetoken: Timetoken::default(),
            client: Option::default(),
            subscribe_key: String::default(),
            flags: Default::default(),
        }
    }
}

impl Default for Timetoken {
    #[must_use]
    fn default() -> Self {
        Self {
            t: "0".to_string(),
            r: 0,
        }
    }
}

impl std::fmt::Display for Timetoken {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "Timetoken {{ t: {}, r: {} }}", self.t, self.r)
    }
}

/// # Send a publish request and return the JSON response
async fn publish_request(http_client: &HttpClient, url: Uri) -> Result<Timetoken, Error> {
    // Send network request
    let res = http_client.get(url).await;
    let mut body = res.unwrap().into_body();
    let mut bytes = Vec::new();

    // Receive the response as a byte stream
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk?);
    }

    // Convert the resolved byte stream to JSON
    let data = std::str::from_utf8(&bytes)?;
    let data_json = json::parse(data)?;
    let timetoken = Timetoken {
        t: data_json[2].to_string(),
        r: 0, // TODO
    };

    // Deliever the timetoken response from PubNub
    Ok(timetoken)
}

/// # Send a subscribe request and return the JSON messages received
async fn subscribe_request(
    http_client: &HttpClient,
    url: Uri,
) -> Result<(Vec<Message>, Timetoken), Error> {
    // Send network request
    let res = http_client.get(url).await;
    let mut body = res.unwrap().into_body();
    let mut bytes = Vec::new();

    // Receive the response as a byte stream
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk?);
    }

    // Convert the resolved byte stream to JSON
    let data = std::str::from_utf8(&bytes)?;
    let data_json = json::parse(data)?;

    // Decode the stream timetoken
    let timetoken = Timetoken {
        t: data_json["t"]["t"].to_string(),
        r: data_json["t"]["r"].as_u32().unwrap_or(0),
    };

    // Capture Messages in Vec Buffer
    let messages = data_json["m"]
        .members()
        .map(|message| Message {
            message_type: MessageType::from_json(&message["e"]),
            route: message["b"].as_str().map(ToString::to_string),
            channel: message["c"].to_string(),
            json: message["d"].clone(),
            metadata: message["u"].clone(),
            timetoken: Timetoken {
                t: message["p"]["t"].to_string(),
                r: message["p"]["r"].as_u32().unwrap_or(0),
            },
            client: message["i"].as_str().map(ToString::to_string),
            subscribe_key: message["k"].to_string(),
            flags: message["f"].as_u32().unwrap_or(0),
        })
        .collect::<Vec<_>>();

    // Deliver the message response from PubNub
    Ok((messages, timetoken))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::current_thread::Runtime;

    fn init() {
        let env = env_logger::Env::default().default_filter_or("pubnub=trace");
        let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
    }

    #[test]
    fn pubnub_subscribe_ok() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let agent = "Rust-Agent-Test";

            let mut pubnub = PubNubBuilder::new(publish_key, subscribe_key)
                .agent(agent)
                .build();

            {
                // Create a subscription
                let mut subscription = pubnub.subscribe(channel).await;
                assert_eq!(
                    subscription.name,
                    ListenerType::Channel(channel.to_string())
                );

                // Send a message to it
                let message = JsonValue::String("Hello, world!".to_string());
                debug!("Publishing...");
                let status = pubnub.publish(channel, message).await;
                assert!(status.is_ok());

                // Receive the message
                debug!("Waiting for message...");
                let message = subscription.next().await;
                assert!(message.is_some());

                // Check the message contents
                let message = message.unwrap();
                assert_eq!(message.message_type, MessageType::Publish);
                let expected = JsonValue::String("Hello, world!".to_string());
                assert_eq!(message.json, expected);
                assert_eq!(message.timetoken.t.len(), 17);
                assert!(message.timetoken.t.chars().all(|c| c >= '0' && c <= '9'));

                debug!("Going to drop Subscription...");
            }
            debug!("Subscription should have been dropped by now");

            debug!("SubscribeLoop should be stopping...");
            match pubnub.pipe.as_mut().unwrap().rx.next().await {
                Some(PipeMessage::Exit) => (),
                error => panic!("Unexpected message: {:?}", error),
            }

            debug!("SubscribeLoop should have stopped by now");
        });
    }

    #[test]
    fn pubnub_subscribeloop_drop() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let mut pubnub = PubNub::new(publish_key, subscribe_key);

            {
                // Create a bunch of subscriptions
                let _sub0 = pubnub.subscribe(channel).await;
                let _sub1 = pubnub.subscribe(channel).await;
                let _sub2 = pubnub.subscribe(channel).await;
                let _sub3 = pubnub.subscribe(channel).await;
                let _sub4 = pubnub.subscribe(channel).await;
                let _sub5 = pubnub.subscribe(channel).await;
                let _sub6 = pubnub.subscribe(channel).await;
                let _sub7 = pubnub.subscribe(channel).await;
                let _sub8 = pubnub.subscribe(channel).await;
                let _sub9 = pubnub.subscribe(channel).await;
                let _sub10 = pubnub.subscribe(channel).await;
                let _sub11 = pubnub.subscribe(channel).await;

                // HA-HAAAA! Now we drop 12 at once and see if the `Drop` impl hangs!
            }

            debug!("SubscribeLoop should be stopping...");
            match pubnub.pipe.as_mut().unwrap().rx.next().await {
                Some(PipeMessage::Exit) => (),
                error => panic!("Unexpected message: {:?}", error),
            }
            debug!("SubscribeLoop should have stopped by now");
        });
    }

    #[test]
    fn pubnub_subscribeloop_recreate() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let mut pubnub = PubNub::new(publish_key, subscribe_key);

            // Create two subscribe loops, dropping each
            {
                let _ = pubnub.subscribe(channel).await;
            }
            match pubnub.pipe.as_mut().unwrap().rx.next().await {
                Some(PipeMessage::Exit) => (),
                error => panic!("Unexpected message: {:?}", error),
            }

            {
                let _ = pubnub.subscribe(channel).await;
            }
            match pubnub.pipe.as_mut().unwrap().rx.next().await {
                Some(PipeMessage::Exit) => (),
                error => panic!("Unexpected message: {:?}", error),
            }
        });
    }

    #[test]
    fn pubnub_publish_ok() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo";

            let agent = "Rust-Agent-Test";

            let pubnub = PubNubBuilder::new(publish_key, subscribe_key)
                .agent(agent)
                .build();

            assert_eq!(pubnub.agent, agent);
            assert_eq!(pubnub.subscribe_key, subscribe_key);
            assert_eq!(pubnub.publish_key, publish_key);

            let message = JsonValue::String("Hi!".to_string());
            let status = pubnub.publish(channel, message).await;
            assert!(status.is_ok());
            let timetoken = status.unwrap();

            assert_eq!(timetoken.t.len(), 17);
            assert!(timetoken.t.chars().all(|c| c >= '0' && c <= '9'));
        });
    }
}
