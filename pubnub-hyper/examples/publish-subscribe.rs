#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures_util::stream::StreamExt;
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::{core::json::object, Builder};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = Hyper::new()
        .publish_key("demo")
        .subscribe_key("demo")
        .build()
        .unwrap();

    let mut pubnub = Builder::new()
        .transport(transport)
        .runtime(TokioGlobal)
        .build();

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
