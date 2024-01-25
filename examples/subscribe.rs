use std::collections::HashMap;

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
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_transport()
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

    client
        .set_presence_state(HashMap::<String, String>::from([
            (
                "is_doing".to_string(),
                "Nothing... Just hanging around...".to_string(),
            ),
            ("flag".to_string(), "false".to_string()),
        ]))
        .channels(["my_channel".into(), "other_channel".into()].to_vec())
        .user_id("user_id")
        .execute()
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let subscription = client.subscription(SubscriptionParams {
        channels: Some(&["my_channel", "other_channel"]),
        channel_groups: None,
        options: Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
    });
    subscription.subscribe(None);
    let subscription_clone = subscription.clone_empty();

    // Attach connection status to the PubNub client instance.
    tokio::spawn(
        client
            .status_stream()
            .for_each(|status| async move { println!("\nstatus: {:?}", status) }),
    );

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

    tokio::spawn(subscription_clone.stream().for_each(|event| async move {
        match event {
            Update::Message(message) | Update::Signal(message) => {
                // Deserialize the message payload as you wish
                match serde_json::from_slice::<Message>(&message.data) {
                    Ok(message) => println!("(b) defined message: {:?}", message),
                    Err(_) => {
                        println!("(b) other message: {:?}", String::from_utf8(message.data))
                    }
                }
            }
            Update::Presence(presence) => {
                println!("(b) presence: {:?}", presence)
            }
            Update::AppContext(object) => {
                println!("(b) object: {:?}", object)
            }
            Update::MessageAction(action) => {
                println!("(b) message action: {:?}", action)
            }
            Update::File(file) => {
                println!("(b) file: {:?}", file)
            }
        }
    }));

    // Sleep for a minute. Now you can send messages to the channels
    // "my_channel" and "other_channel" and see them printed in the console.
    // You can use the publish example or [PubNub console](https://www.pubnub.com/docs/console/)
    // to send messages.
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    // You can also cancel the subscription at any time.
    // subscription.unsubscribe();

    println!("\nDisconnect from the real-time data stream");
    client.disconnect();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("\nReconnect to the real-time data stream");
    client.reconnect(None);

    // Let event engine process unsubscribe request
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // If Subscription or Subscription will go out of scope they will unsubscribe.
    // drop(subscription);
    // drop(subscription_clone);

    println!(
        "\nUnsubscribe from all data streams. To restore requires `subscription.subscribe(None)` call." );
    // Clean up before complete work with PubNub client instance.
    client.unsubscribe_all();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
