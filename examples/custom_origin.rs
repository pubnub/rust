use pubnub::{transport::TransportReqwest, Keyset, PubNubClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn snafu::Error>> {
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let transport = {
        let mut transport = TransportReqwest::default();

        // this is the default server, change it to your custom origin
        transport.set_hostname("https://ps.pndsn.com");

        transport
    };

    let client = PubNubClientBuilder::with_transport(transport)
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    // publish to check the custom origin
    let result = client
        .publish_message("hello world!")
        .channel("my_channel")
        .custom_message_type("text-message")
        .execute()
        .await?;

    println!("publish result: {:?}", result);

    Ok(())
}
