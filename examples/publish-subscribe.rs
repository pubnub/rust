#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures_util::stream::StreamExt;
use pubnub::{json::object, StandardPubNub as PubNub};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pubnub = PubNub::new("demo", "demo");

    let message = object! {
        "username" => "JoeBob",
        "content" => "Hello, world!",
    };

    let mut stream = pubnub.subscribe("my-channel").await;
    let timetoken = pubnub.publish("my-channel", message).await?;
    println!("timetoken = {:?}", timetoken);

    let received = stream.next().await;
    println!("received = {:?}", received);

    Ok(())
}
