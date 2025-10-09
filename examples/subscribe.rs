use futures::StreamExt;
use serde::Deserialize;
use std::env;

use pubnub::subscribe::{SubscriptionOptions, SubscriptionParams};
use pubnub::{
    dx::subscribe::Update,
    subscribe::{EventEmitter, EventSubscriber},
    Keyset, PubNubClientBuilder,
};

#[derive(Debug, Deserialize)]
struct Message {
    // Allowing dead code because we don't use these fields
    // in this example.
    #[allow(dead_code)]
    url: String,
    #[allow(dead_code)]
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn snafu::Error>> {
    env_logger::init();
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let pubnub = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .with_filter_expression("some_filter")
        .with_heartbeat_value(100)
        .with_heartbeat_interval(5)
        .build()?;

    println!("running!");

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let subscription = pubnub.subscription(SubscriptionParams {
        channels: Some(&["my_channel", "other_channel"]),
        channel_groups: None,
        options: Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
    });
    subscription.subscribe();
    let subscription_clone = subscription.clone_empty();

    // Attach connection status to the PubNub client instance.
    tokio::spawn(
        pubnub
            .status_stream()
            .for_each(|status| async move { println!("\nstatus: {:?}", status) }),
    );

    // Example of the "global" listener for multiplexed subscription object from
    // PubNub client.
    tokio::spawn(subscription.stream().for_each(|event| async move {
        match event {
            Update::Message(message) | Update::Signal(message) => {
                // Deserialize the message payload as you wish
                match serde_json::from_slice::<Message>(&message.data) {
                    Ok(message) => println!("(a) defined message: {:?}", message),
                    Err(_) => {
                        println!("(a) other message: {:?}", String::from_utf8(message.data))
                    }
                }
            }
            Update::Presence(presence) => {
                println!("(a) presence: {:?}", presence)
            }
            Update::AppContext(object) => {
                println!("(a) object: {:?}", object)
            }
            Update::MessageAction(action) => {
                println!("(a) message action: {:?}", action)
            }
            Update::File(file) => {
                println!("(a) file: {:?}", file)
            }
        }
    }));

    // Explicitly listen only for real-time `message` updates.
    tokio::spawn(
        subscription_clone
            .messages_stream()
            .for_each(|message| async move {
                // Deserialize the message payload as you wish
                match serde_json::from_slice::<Message>(&message.data) {
                    Ok(message) => println!("(b) defined message: {:?}", message),
                    Err(_) => {
                        println!("(b) other message: {:?}", String::from_utf8(message.data))
                    }
                }
            }),
    );

    // Explicitly listen only for real-time `file` updates.
    tokio::spawn(
        subscription_clone
            .files_stream()
            .for_each(|file| async move { println!("(b) file: {:?}", file) }),
    );
    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    println!("\nWaiting for message");
    //
    // println!("\nPublish a message:");
    //
    // pubnub
    //     .publish_message("{\"event\":\"flag_changed\",\"timestamp\":
    // 1758130367173}")     .channel("my_channel")
    //     .execute()
    //     .await?;
    // println!("\nDone!");

    // Sleep for a minute. Now you can send messages to the channels
    // "my_channel" and "other_channel" and see them printed in the console.
    // You can use the publishing example or [PubNub console](https://www.pubnub.com/docs/console/)
    // to send messages.
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    // You can also cancel the subscription at any time.
    // subscription.unsubscribe();

    println!("\nDisconnect from the real-time data stream");
    pubnub.disconnect();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("\nReconnect to the real-time data stream");
    pubnub.reconnect(None);

    // Let event engine process unsubscribe request
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // If Subscription or Subscription will go out of scope they will unsubscribe.
    // drop(subscription);
    // drop(subscription_clone);

    println!(
        "\nUnsubscribe from all data streams. To restore call `subscription.subscribe()` or \
        `subscription.subscribe_with_timetoken(Some(<timetoken>)) call."
    );
    // Clean up before complete work with PubNub client instance.
    pubnub.unsubscribe_all();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
