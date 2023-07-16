use pubnub::{Keyset, PubNubClientBuilder};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Message {
    url: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn snafu::Error>> {
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let _client = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    //client
    //    .subscribe_with_spawner()
    //    .channels(["hello".into(), "world".into()].to_vec())
    //    .heartbeat(10)
    //    .filter_expression("some_filter".into())
    //    .build()?
    //    .stream()
    //    .for_each(|message| async {
    //        println!("message: {:?}", message);
    //    })
    //    .await;

    // TODO: something like that
    // let stream = subscription.stream();
    // tokio::spawn(async move {
    //      stream.then(|message| {
    //      println!("message: {:?}", message);
    //   }).await;
    //
    //   println!("stream cancelled!");
    // };
    // let mut subscription = client.subscribe().build().unwrap();

    //subscription.unsubscribe().await;
    //    subscription
    //        .for_each(|updates| async move {
    //            updates.iter().for_each(|update| match update {
    //                SubscribeStreamEvent::Status(status) => println!("Status changed: {status:?}"),
    //                SubscribeStreamEvent::Update(update) => println!("Received update: {update:?}"),
    //            })
    //        })
    //        .await;

    Ok(())
}
