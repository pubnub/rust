use pubnub::{Keyset, PubNubClientBuilder};
use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
struct State {
    is_doing: String,
}

fn main() -> Result<(), Box<dyn snafu::Error>> {
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_blocking_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    println!("running!");

    client
        .set_presence_state(State {
            is_doing: "Nothing... Just hanging around...".into(),
        })
        .channels(["my_channel".into(), "other_channel".into()].to_vec())
        .user_id("user_id")
        .execute_blocking()?;

    println!("State set!");
    println!();

    let states = client
        .get_presence_state()
        .channels(["my_channel".into(), "other_channel".into()].to_vec())
        .user_id("user_id")
        .execute_blocking()?;

    println!("All channels state: {:?}", states);

    states.iter().for_each(|channel| {
        println!("Channel: {}", channel.channel);
        println!("State: {:?}", channel.state);
    });

    Ok(())
}
