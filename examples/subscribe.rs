use futures::StreamExt;
use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
use pubnub::{Keyset, PubNubClientBuilder};
use serde::Deserialize;
use std::env;

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
        .build()?;

    println!("running!");

    client
        .subscribe()
        .channels(["hello".into(), "world".into()].to_vec())
        .heartbeat(10)
        .filter_expression("some_filter")
        .execute()?
        .stream()
        .for_each(|event| async move {
            match event {
                SubscribeStreamEvent::Update(update) => {
                    println!("update: {:?}", update);
                    match update {
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
                        Update::Object(object) => {
                            println!("object: {:?}", object)
                        }
                        Update::MessageAction(action) => {
                            println!("message action: {:?}", action)
                        }
                        Update::File(file) => {
                            println!("file: {:?}", file)
                        }
                    }
                }
                SubscribeStreamEvent::Status(status) => println!("status: {:?}", status),
            }
        })
        .await;

    Ok(())
}
