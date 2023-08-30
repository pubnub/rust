use pubnub::{Keyset, PubNubClientBuilder};
use std::env;

fn main() -> Result<(), Box<dyn snafu::Error>> {
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

    println!("running!");

    //    let where_user = client
    //        .where_now()
    //        .user_id("user_id")
    //        .execute_blocking()?;
    //
    //    println!("All channels data: {:?}", where_user);

    Ok(())
}
