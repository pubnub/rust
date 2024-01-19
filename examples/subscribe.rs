use std::collections::HashMap;

use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use std::env;

use pubnub::subscribe::SubscriptionOptions;
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
    let publish_key = "demo"; //env::var("SDK_PUB_KEY")?;
    let subscribe_key = "demo"; //env::var("SDK_SUB_KEY")?;

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

    let subscription = client.subscription(
        Some(&["my_channel", "other_channel"]),
        None,
        Some(vec![SubscriptionOptions::ReceivePresenceEvents]),
    );
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
                    Ok(message) => println!("defined message: {:?}", message),
                    Err(_) => {
                        println!("other message: {:?}", String::from_utf8(message.data))
                    }
                }
            }
            Update::Presence(presence) => {
                println!("presence: {:?}", presence)
            }
            Update::AppContext(object) => {
                println!("object: {:?}", object)
            }
            Update::MessageAction(action) => {
                println!("message action: {:?}", action)
            }
            Update::File(file) => {
                println!("file: {:?}", file)
            }
        }
    }));

    tokio::spawn(subscription_clone.stream().for_each(|event| async move {
        match event {
            Update::Message(message) | Update::Signal(message) => {
                // Deserialize the message payload as you wish
                match serde_json::from_slice::<Message>(&message.data) {
                    Ok(message) => println!("~~~~~> defined message: {:?}", message),
                    Err(_) => {
                        println!("other message: {:?}", String::from_utf8(message.data))
                    }
                }
            }
            Update::Presence(presence) => {
                println!("~~~~~> presence: {:?}", presence)
            }
            Update::AppContext(object) => {
                println!("~~~~~> object: {:?}", object)
            }
            Update::MessageAction(action) => {
                println!("~~~~~> message action: {:?}", action)
            }
            Update::File(file) => {
                println!("~~~~~> file: {:?}", file)
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

    println!("~~~~~~~~> DISCONNECT");
    client.disconnect();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("~~~~~~~~> RECONNECT");
    client.reconnect(None);

    // Let event engine process unsubscribe request
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("~~~~~~~~> UNSUBSCRIBE ALL...");

    // Clean up before complete work with PubNub client instance.
    client.unsubscribe_all();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("~~~~~~~~> UNSUBSCRIBE ALL. DONE");

    Ok(())
}
