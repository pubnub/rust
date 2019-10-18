extern crate hyper;
extern crate uuid;

use log::debug; // debug!(...);
use uuid::Uuid;
use std::io::{self, Write};
use tokio::sync::mpsc;
use hyper::rt::{self, Future, Stream};
//use std::collections::HashMap;


// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Client
///
/// This is the structure that is used to add and remove client connections
/// for channels and channel groups using additional parameters for filtering.
/// The `userID` is the same as the UUID used in PubNub SDKs.
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct Client {
    pub publish_key   : String, // Customer's Publish Key
    pub subscribe_key : String, // Customer's Subscribe Key
    pub secret_key    : String, // Customer's Secret Key
    pub auth_key      : String, // Client Auth Key for R+W Access
    pub channels      : String, // Client Channels Comma Separated
    pub groups        : String, // Client Channel Groups Comma Sepparated
    pub user_id       : String, // Client UserId "UUID" for Presence
    pub filters       : String, // Metadata Filters on Messages
    pub presence      : bool,   // Enable presence events
    pub agent         : String, // "Rust-Generic"
    pub since         : u64,    // Unix Timestamp Fetch History + Subscribe
    timetoken         : String, // Current Queue Line-in-Sand for Subscription
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Message Types
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub enum MessageType {
    Publish,     // Response of Publish (Success/Fail)
    Subscribe,   // Response of Subscription ( Usually a Message Payload )
    Presence,    // Presence Event from Channel ( Another Client Joined )
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Message
///
/// This is the message structure that includes all known information on the
/// message received via `pubnub.next()`.
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct Message {
    pub client       : Client,      // Copy of Client - for pubnub.remove() 
    pub message_type : MessageType, // Enum Type of Message
    pub channel      : String,      // Origin Channel of Message Receipt
    pub data         : String,      // Data Payload of Channel
    pub metadata     : String,      // Metadata of Message
    pub timetoken    : String,      // Message ID Timetoken
    pub success      : bool,        // Useful to see if Publish was Successful
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub
///
/// The PubNub lib implements socket pools to relay data requests as a client
/// connection to the PubNub Network.
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct PubNub {
    origin: String, // "ps.pndsn.com:443"
    // vec of 
    //requests: Hyper,
    // hyper client ( use .clone to add new pool entry )
    // some sort of connection ???
    // list of connected clients + lookup concat(pubkey,subkey,channels,uuid,auth_key)
    // mpsc things
}

#[derive(Debug)]
pub enum Error {
    Initialize,
    Publish,
    Subscribe,
    Ping,
    Exit,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Pool
///
/// This client lib offers publish and subscribe support to PubNub.
/// Additionally creates an upstream pool and maintains connectivity for
/// thousands of clients.  Client count limited to machine resources.
/// Autoscale resources as needed.
///
/// This is the base structure which creates two threads for
/// Publish and Subscribe.
///
/// ```
/// use PubNub, Client, Message, MessageType;
///
/// let channels      = "demo,demolition-man";
/// let publish_key   = "demo";
/// let subscribe_key = "demo";
/// let secret_key    = "";
/// let origin        = "ps.pndsn.com:443";
/// let agent         = "Rust-Agent";
/// 
/// let mut pubnub = PubNub::new(
///     origin : &origin,
/// ).expect("Failed to create PubNub.");
/// 
/// let mut client = Client::new(
///     publish_key   : publish_key,
///     subscribe_key : subscribe_key,
///     secret_key    : secret_key,
///     channels      : channels,
///     groups        : "",
///     userId        : "12345",
///     auth_key      : "",
///     filters       : "",
///     presence      : true,
///     timetoken     : "0",
/// );
/// 
/// pubnub.add(client);
/// client.publish("demo", "demo");
/// 
/// while let Some(message) = pubnub.next().await {
///     // TODO Match on MessageType match message.message_type {}
///     // Print message and channel name.
///     println!("{}: {}", message.channel, message.data);
///     
///     // Remove clients only when you no longer need them
///     // When no more clients are in the pool, then `pubnub.next()` will
///     // return `None` and the loop will exit.
///     pubnub.remove(message.client);
/// }
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl PubNub {
    pub fn new(origin : Option<&str>) -> Result<PubNub, Error> {
        // TODO Start mpsc tokyo things
        // TODO Subscribe tokyo things
        // TODO Publish tokyo things
        Ok(PubNub {
            origin : origin.unwrap_or("ps.pndsn.com:443").to_string(),
        })
    }
    pub fn add(self, client: Client) {}
    pub fn remove(self, client: Client) {}
    // https://github.com/actix/examples/blob/master/http-proxy/src/main.rs
    // https://docs.rs/futures-preview/0.3.0-alpha.18/futures/stream/trait.Stream.html
    pub fn next(self) {}
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Publish/Subscribe Client
///
/// This client lib offers publish must be added to PubNub
/// for both Publish and Subscribe.
///
/// ```
/// use PubNub, Client, Message, MessageType;
///
/// let channels      = "demo,demolition-man";
/// let publish_key   = "demo";
/// let subscribe_key = "demo";
/// let auth_key      = "";
/// let origin        = "ps.pndsn.com:443";
/// let agent         = "Rust-Agent";
/// 
/// let mut pubnub = PubNub::new(
///     origin : &origin,
/// ).expect("Failed to create PubNub.");
/// 
/// let mut client = Client::new(
///     publish_key   : publish_key,
///     subscribe_key : subscribe_key,
///     auth_key      : auth_key,
/// );
/// 
/// pubnub.add(client);
/// client.publish("demo", "demo");
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Client {
    pub fn new(
        subscribe_key : String,
        publish_key   : Option<&str>,
        secret_key    : Option<&str>,
        auth_key      : Option<&str>,
        channels      : Option<&str>, // subscription channels
        groups        : Option<&str>, // subscription channel groups
        filters       : Option<&str>, // subscription filters
        presence      : Option<bool>, // enable presence events
        user_id       : Option<&str>,
        agent         : Option<&str>,
        since         : Option<u64>, // TODO
        timetoken     : Option<&str>,
    ) -> Result<Client, Error> {
        // TODO Start mpsc threads NO - can not have 1 thread per client...
        // Maybe we can have it dedicated to a PubNub pool...
        // AH!! Gets a clone() of the mpsc sender for PubNub for publishing.

        // TODO
        //let default_user_id = Uuid::new_v4().hyphenated();

        Ok(Client {
            subscribe_key : subscribe_key.to_string(),
            publish_key   : publish_key.unwrap_or("demo").to_string(),
            secret_key    : secret_key.unwrap_or("demo").to_string(),
            auth_key      : auth_key.unwrap_or("").to_string(),
            channels      : channels.unwrap_or(",").to_string(),
            groups        : groups.unwrap_or("").to_string(),
            user_id       : user_id.unwrap_or("").to_string(),
            filters       : filters.unwrap_or("").to_string(),
            presence      : presence.unwrap_or(false),
            agent         : agent.unwrap_or("Rust-Agent").to_string(),
            since         : since.unwrap_or(0),
            timetoken     : timetoken.unwrap_or("0").to_string(),
        })
    }

    pub fn publish(self, channel: &str, data: &str, metadata: Option<&str>) {
        // sends mpsc to a loop generated in new()
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests for PubNub Pool
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pubnub_time_ok() {
        let host = "0.0.0.0:3000";
        assert!(true);
        assert!(true);
    }

    #[test]
    async fn pubnub_publish_ok() {
        let origin        = "ps.pndsn.com:443";
        let publish_key   = "demo";
        let subscribe_key = "demo";
        let channel       = "demo";
        let agent         = "Rust-Agent-Test";

        let mut pubnub = PubNub::new(
            origin : origin,
        ).expect("Failed to create PubNub.");

        let mut client = Client::new(
            publish_key   : publish_key,
            subscribe_key : Some(subscribe_key),
            channels      : Some(channels),
            secret_key    : None,
            user_id       : None,
            auth_key      : None,
            filters       : None,
            presence      : None,
            timetoken     : None,
        );

        pubnub.add(client);
        client.publish("demo", "demo", None);

        while let Some(message) = pubnub.next().await {
            // TODO Match on MessageType match message.message_type {}
            // Print message and channel name.
            println!("{}: {}", message.channel, message.data);
            
            // Remove clients only when you no longer need them
            // When no more clients are in the pool, then `pubnub.next()` will
            // return `None` and the loop will exit.
            pubnub.remove(message.client);
        }
    }
}
