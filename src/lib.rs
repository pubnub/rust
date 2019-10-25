//! # Async PubNub Client SDK for Rust
//!
//! - Fully `async`/`await` ready.
//! - Uses Tokio and Hyper to provide an ultra-fast, incredibly reliable message transport over the
//! PubNub edge network.
//! - Optimizes for minimal network sockets with an infinite number of logical streams.

use std::collections::HashMap;
use std::time::Duration;

use futures_util::future::AbortHandle;
use hyper::{client::HttpConnector, Uri};
use hyper_tls::HttpsConnector;
use json::JsonValue;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use thiserror::Error;
use tokio::sync::mpsc;

type HttpClient = hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>;
type Channel = mpsc::Sender<Message>;

/// # PubNub Client
///
/// The PubNub lib implements socket pools to relay data requests as a client connection to the
/// PubNub Network.
#[derive(Debug, Clone)]
pub struct PubNub {
    origin: String,                      // "domain:port"
    agent: String,                       // "Rust-Agent"
    client: HttpClient,                  // HTTP Client
    publish_key: String,                 // Customer's Publish Key
    subscribe_key: String,               // Customer's Subscribe Key
    secret_key: Option<String>,          // Customer's Secret Key
    auth_key: Option<String>,            // Client Auth Key for R+W Access
    user_id: Option<String>,             // Client UserId "UUID" for Presence
    filters: Option<String>,             // Metadata Filters on Messages
    presence: bool,                      // Enable presence events
    channels: HashMap<String, Channel>,  // Client Channels
    groups: HashMap<String, Channel>,    // Client Channel Groups
    subscribe_loop: Option<AbortHandle>, // Handles message reception from PubNub
    encoded_channels: String,            // Client Channels, comma-separated and URI encoded
    encoded_groups: String,              // Client Channel Groups, comma-separated and URI encoded
    timetoken: Timetoken,                // Current Line-in-Sand for Subscription
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
#[derive(Debug, Default, Clone)]
pub struct Timetoken {
    t: String, // Timetoken
    r: u32,    // Origin region
}

/// # PubNub Message
///
/// This is the message structure that includes all known information on the message received via
/// `pubnub.next()`.
#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: MessageType, // Enum Type of Message
    pub requested_channel: String, // Wildcard channel or channel group
    pub channel: String,           // Origin Channel of Message Receipt
    pub json: JsonValue,           // Decoded JSON Message Payload
    pub metadata: String,          // Metadata of Message
    pub timetoken: Timetoken,      // Message ID Timetoken
    pub client: String,            // Issuing client ID
    pub subscribe_key: String,     // As if you don't know your own subscribe key!
    pub shard: u32,                // LOL why does the PubNub service provide this?!
    pub flags: u32,                // Your guess is as good as mine!
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Message Types
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageType {
    Publish,  // Response of Publish (Success/Fail)
    Signal,   // A Lightweight message
    Objects,  // An Objects service event, like space description updated
    Action,   // A message action event
    Presence, // Presence Event from Channel ( Another Client Joined )
}

// XXX: These are not ideal...

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Error variants
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Debug, Error)]
pub enum Error {
    #[error("Hyper client error")]
    HyperError(#[source] hyper::Error),

    #[error("Invalid UTF-8")]
    Utf8Error(#[source] std::str::Utf8Error),

    #[error("Invalid JSON")]
    JsonError(#[source] json::Error),
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::HyperError(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error::Utf8Error(error)
    }
}

impl From<json::Error> for Error {
    fn from(error: json::Error) -> Error {
        Error::JsonError(error)
    }
}

/// # PubNub Tokio Runtime w/ Hyper Worker
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
    /// # Create a new `PubNub` client with default configuration.
    ///
    /// To create a `PubNub` client with custom configuration, use [`PubNubBuilder::new`].
    pub fn new(publish_key: &str, subscribe_key: &str) -> PubNub {
        PubNubBuilder::new(publish_key, subscribe_key).build()
    }

    /// # Set the subscribe filters.
    ///
    /// ```no_run
    /// # use pubnub::PubNub;
    /// let mut pubnub = PubNub::new("demo", "demo");
    /// pubnub.filters("uuid != JoeBob");
    /// ```
    pub fn filters(&mut self, filters: &str) {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
    }

    /// # Publish a message over the PubNub network.
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

    /// # Publish a message over the PubNub network with an extra metadata payload.
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

    /// # Encode the internal channel list to a string.
    ///
    /// This is also used for encoding the list of channel groups.
    fn encode_channels(&self, channels: &HashMap<String, Channel>) -> String {
        channels
            .keys()
            .map(|channel| utf8_percent_encode(channel, NON_ALPHANUMERIC).to_string())
            .collect::<Vec<_>>()
            .as_slice()
            .join("%2C")
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
    /// # Create a new `PubNubBuilder` that can configure a `PubNub` client.
    pub fn new(publish_key: &str, subscribe_key: &str) -> PubNubBuilder {
        PubNubBuilder {
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

    /// # Set the PubNub network origin.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .origin("pubsub.pubnub.com")
    ///     .build();
    /// ```
    pub fn origin(mut self, origin: &str) -> PubNubBuilder {
        self.origin = origin.to_string();
        self
    }

    /// # Set the HTTP user agent string.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .agent("My Awesome Rust App/1.0.0")
    ///     .build();
    /// ```
    pub fn agent(mut self, agent: &str) -> PubNubBuilder {
        self.agent = agent.to_string();
        self
    }

    /// # Set the PubNub secret key.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .secret_key("sub-c-deadbeef-0000-1234-abcd-c0deface")
    ///     .build();
    /// ```
    pub fn secret_key(mut self, secret_key: &str) -> PubNubBuilder {
        self.secret_key = Some(secret_key.to_string());
        self
    }

    /// # Set the PubNub PAM auth key.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .auth_key("Open-Sesame!")
    ///     .build();
    /// ```
    pub fn auth_key(mut self, auth_key: &str) -> PubNubBuilder {
        self.auth_key = Some(auth_key.to_string());
        self
    }

    /// # Set the PubNub User ID (Presence UUID).
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .user_id("JoeBob")
    ///     .build();
    /// ```
    pub fn user_id(mut self, user_id: &str) -> PubNubBuilder {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// # Set the subscribe filters.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .filters("uuid != JoeBob")
    ///     .build();
    /// ```
    pub fn filters(mut self, filters: &str) -> PubNubBuilder {
        self.filters = Some(utf8_percent_encode(filters, NON_ALPHANUMERIC).to_string());
        self
    }

    /// # Enable or disable interest in receiving Presence events.
    ///
    /// When enabled (default), `pubnub.next()` will provide messages with `MessageType::Presence`
    /// when users join and leave the channels you are listening on.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .presence(true)
    ///     .build();
    /// ```
    pub fn presence(mut self, enable: bool) -> PubNubBuilder {
        self.presence = enable;
        self
    }

    /// # Build the PubNub client to begin streaming messages.
    ///
    /// ```no_run
    /// # use pubnub::PubNubBuilder;
    /// let pubnub = PubNubBuilder::new("demo", "demo")
    ///     .build();
    /// ```
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
            channels: HashMap::new(),
            groups: HashMap::new(),
            subscribe_loop: None,
            encoded_channels: String::new(),
            encoded_groups: String::new(),
            timetoken: Timetoken::default(),
        }
    }
}

impl MessageType {
    /// # Create a `MessageType` from an integer.
    ///
    /// Subscribe message pyloads include a non-enumerated integer to describe message types. We
    /// instead provide a concrete type, using this function to convert the integer into the
    /// appropriate type.
    fn from_json(i: JsonValue) -> MessageType {
        if let Some(i) = i.as_u32() {
            match i {
                0 => MessageType::Publish,
                1 => MessageType::Signal,
                2 => MessageType::Objects,
                3 => MessageType::Action,
                _ => panic!("Invalid message type: {}", i),
            }
        } else {
            panic!("Invalid message type: {}", i);
        }
    }
}

/*
    {
        let origin = "ps.pndsn.com";
        let (submit_publish, mut process_publish) = mpsc::channel::<PublishMessage>(100);
        let (submit_subscribe, mut process_subscribe) = mpsc::channel::<Client>(100);
        let (submit_result, process_result) = mpsc::channel::<Message>(100);

        let https = HttpsConnector::new().unwrap();
        let http_client = hyper::Client::builder()
            .keep_alive_timeout(Some(Duration::from_secs(300)))
            .max_idle_per_host(10000)
            .build::<_, hyper::Body>(https);
        let subscribe_http_client = http_client.clone();

        // Start Subscribe Worker
        // Messages available via pubnub.next()
        let mut subscribe_result = submit_result.clone();
        let mut resubmit_subscribe = submit_subscribe.clone();
        tokio::spawn(async move {
            // TODO loop the subscribe or it wont work.
            // TODO save timetoken
            while let Some(mut client) = process_subscribe.recv().await {
                // Construct URI
                let url = format!(
                    "https://{origin}/v2/subscribe/{sub_key}/{channels}/0/{timetoken}",
                    origin = origin.to_string(),
                    sub_key = client.subscribe_key,
                    channels = client.channels,
                    timetoken = client.timetoken,
                );

                // Send network request
                let url = url.parse().expect("Unable to parse URL");
                let (messages, timetoken) = subscribe_request(&subscribe_http_client, url)
                    .await
                    .expect("TODO: Handle errors gracefully!");

                // Save Timetoken for next request
                client.timetoken = timetoken;

                // Submit another subscribe event to be processed
                // TODO handle errors
                match resubmit_subscribe.try_send(client.clone()) {
                    Ok(()) => {}      //Ok(()),
                    Err(_error) => {} //Err(Error::SubscribeChannelWrite(error)),
                }

                // Result Message from PubNub
                for message in messages {
                    // Send Subscription Result to End-user via MPSC
                    // User can recieve subscription messages via pubnub.next()
                    // TODO handle errors
                    match subscribe_result.try_send(message) {
                        Ok(()) => {}
                        //Err(error) => {Err(Error::ResultChannelWrite(error));},
                        Err(_error) => {}
                    };
                }
            }
        });

        PubNub {
            origin: "ps.pndsn.com".to_string(), // Change via pubnub.origin()
            agent: "Rust-Agent".to_string(),    // Change via pubnub.agent()
            submit_publish,                     // Publish a Message
            submit_subscribe,                   // Add a Client
            submit_result,                      // Send Result to Application Consumer
            process_result,                     // Receiver for Application Consumer
        }
    }
*/

/// # Send a publish request and return the JSON response.
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

    // Response Message received at pubnub.next()
    Ok(timetoken)
}

/// # Send a subscribe request and return the JSON messages received.
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
            message_type: MessageType::from_json(message["e"].clone()),
            requested_channel: message["b"].to_string(),
            channel: message["c"].to_string(),
            json: message["d"].clone(),
            metadata: message["u"].to_string(),
            timetoken: Timetoken {
                t: message["p"]["t"].to_string(),
                r: message["p"]["r"].as_u32().unwrap_or(0),
            },
            client: message["i"].to_string(),
            subscribe_key: message["k"].to_string(),
            shard: message["a"].as_u32().unwrap_or(0),
            flags: message["f"].as_u32().unwrap_or(0),
        })
        .collect::<Vec<_>>();

    // Result Message from PubNub
    Ok((messages, timetoken))
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests for PubNub Pool
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    /*
            #[test]
            fn pubnub_time_ok() {
                // TODO
                let _host = "0.0.0.0:3000";
                assert!(true);
                assert!(true);
            }

            #[test]
            fn pubnub_subscribe_ok() {
                let rt = Runtime::new().unwrap();
                let mut exec = rt.executor();
                tokio_executor::with_default(&mut exec, || {
                    let publish_key = "demo";
                    let subscribe_key = "demo";
                    let channels = "demo";
                    let origin = "ps.pndsn.com";
                    let agent = "Rust-Agent-Test";

                    let mut pubnub = PubNub::new()
                        .origin(&origin.to_string())
                        .agent(&agent.to_string());

                    let client = Client::new()
                        .subscribe_key(&subscribe_key)
                        .publish_key(&publish_key)
                        .channels(&channels);

                    let result = pubnub.subscribe(&client);
                    assert!(result.is_ok());

                    let message_future = pubnub.next();
                    let message = rt.block_on(message_future).unwrap();

                    assert!(message.success);
                    /*
                    while let Some(message) = pubnub.next() {

                    }*/
                });
            }
    */

    #[test]
    fn pubnub_publish_ok() {
        let rt = Runtime::new().unwrap();
        let mut exec = rt.executor();
        tokio_executor::with_default(&mut exec, || {
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
            let status_future = pubnub.publish(channel, message);
            let status = rt.block_on(status_future);
            assert!(status.is_ok());
            let timetoken = status.unwrap();

            assert_eq!(timetoken.t.len(), 17);
            assert!(timetoken.t.chars().all(|c| c >= '0' && c <= '9'));

            // rt.block_on(async {
            //     while let Some(message) = pubnub.next().await {
            //         // TODO Match on MessageType match message.message_type {}
            //         // Print message and channel name.
            //         println!("{}: {}", message.channel, message.data);
            //
            //         // Remove clients only when you no longer need them
            //         // When no more clients are in the pool, then `pubnub.next()` will
            //         // return `None` and the loop will exit.
            //         // pubnub.remove(message.client);
            //     }
            // });
        });
    }
}
