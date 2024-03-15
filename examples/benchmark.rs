use futures::stream::StreamExt;
use pubnub::{
    subscribe::{EventEmitter, EventSubscriber, SubscriptionParams},
    Keyset, PubNubClientBuilder,
};
use serde::Serialize;
use std::{
    env,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tokio::time::sleep;

#[derive(Serialize)]
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

    let sub = client.subscription(SubscriptionParams {
        channels: Some(&["kekw"]),
        channel_groups: None,
        options: None,
    });

    sub.subscribe();

    tokio::spawn(sub.messages_stream().for_each(|msg| async move {
        println!(
            "sub {:?}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        println!("Received message: {:?}", msg);
    }));

    println!(
        "pub {:?}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // publish simple string
    client
        .publish_message("chuj dupa cipa")
        .channel("kekw")
        .execute()
        .await?;
    println!(
        "pu2 {:?}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    sleep(Duration::from_secs(1)).await;

    Ok(())
}
