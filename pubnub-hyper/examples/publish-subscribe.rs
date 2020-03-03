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
        .build()?;

    let mut pubnub = Builder::new()
        .transport(transport)
        .runtime(TokioGlobal)
        .build();

    let message = object! {
        "username" => "JoeBob",
        "content" => "Hello, world!",
    };

    let mut stream = pubnub.subscribe("my-channel".parse().unwrap()).await;
    let timetoken = pubnub
        .publish("my-channel".parse().unwrap(), message)
        .await?;
    println!("timetoken = {:?}", timetoken);

    let received = stream.next().await;
    println!("received = {:?}", received);

    Ok(())
}
