use pubnub::{Keyset, PubNubClientBuilder};
use serde::Serialize;
use std::env;

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

    // publish simple string
    let result = client
        .publish_message("hello world!")
        .channel("my_channel")
        .r#type("text-message")
        .execute()
        .await?;

    println!("publish result: {:?}", result);

    // publish with other async task
    let handle = tokio::spawn(
        client
            .publish_message("hello async world!")
            .channel("my_channel")
            .r#type("text-message")
            .execute(),
    );

    // publish a struct
    let result = client
        .publish_message(Message {
            url: "https://this/is/an/example".into(),
            description: "Check out this awesome playlist I made!".into(),
        })
        .channel("my_channel")
        .r#type("url-with-description")
        .execute()
        .await?;

    println!("publish struct result: {:?}", result);

    // publish with all config options
    let result = client
        .publish_message("hello with params!")
        .channel("my_channel")
        .store(true)
        .meta([("meta1".into(), "meta2".into())].into())
        .replicate(true)
        .use_post(true)
        .ttl(10)
        .space_id("my_space")
        .r#type("text-message")
        .execute()
        .await?;

    println!("publish with config result: {:?}", result);

    // unwrap the spawned task and result of the publish
    let result = handle.await??;
    println!("publish async result: {:?}", result);

    Ok(())
}
