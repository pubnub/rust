#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures_util::stream::StreamExt;
use json::object;
use pubnub::{Error, PubNub};

#[tokio::main]
async fn main() -> Result<(), Error> {
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
