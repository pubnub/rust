use pubnub::{Keyset, PubNubClientBuilder};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    content: String,
    author: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publish_key = "demo";
    let subscribe_key = "demo";

    let mut client = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key: subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    // publish simple string
    client
        .publish_message("hello world!")
        .channel("my_channel")
        .execute()
        .await?;

    // publish a struct
    let message = Message {
        content: "hello world!".into(),
        author: "me".into(),
    };

    client
        .publish_message(message)
        .channel("my_channel")
        .execute()
        .await?;

    Ok(())
}
