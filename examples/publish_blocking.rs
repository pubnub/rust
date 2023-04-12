use pubnub::{Keyset, PubNubClientBuilder};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct Message {
    content: String,
    author: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publish_key = env::var("PUBNUB_PUBLISH_KEY")?;
    let subscribe_key = env::var("PUBNUB_SUBSCRIBE_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_blocking_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    // publish simple string
    let result = client
        .publish_message("hello world!")
        .channel("my_channel")
        .execute_blocking()?;

    println!("result: {:?}", result);

    // publish a struct
    let result = client
        .publish_message(Message {
            content: "hello world!".into(),
            author: "me".into(),
        })
        .channel("my_channel")
        .execute_blocking()?;

    println!("result: {:?}", result);

    // publish with whole config options
    let result = client
        .publish_message("hello with params!")
        .channel("my_channel")
        .store(true)
        .meta([("meta1".into(), "meta2".into())].into())
        .replicate(true)
        .use_post(true)
        .ttl(10)
        .space_id("my_space")
        .r#type("my_type")
        .execute_blocking()?;

    println!("result: {:?}", result);

    Ok(())
}
