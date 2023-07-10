use pubnub::{Keyset, PubNubClientBuilder};
use serde::Deserialize;
use spin::rwlock::RwLock;
use std::env;
use std::sync::Arc;

#[derive(Deserialize)]
struct Message {
    url: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn snafu::Error>> {
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    let subscription = client
        .subscribe()
        .channels(vec!["hello_world".to_string()])
        .build()?;

    // TODO: something like that
    // let stream = subscription.stream();
    // tokio::spawn(async move {
    //      stream.then(|message| {
    //      println!("message: {:?}", message);
    //   }).await;
    //
    //   println!("stream cancelled!");
    // };

    subscription.cancel().await?;

    Ok(())
}