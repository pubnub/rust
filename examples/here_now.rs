use pubnub::{Keyset, PubNubClientBuilder};
use std::env;

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

    let channels_now = client
        .here_now()
        .channels(["my_channel".into(), "other_channel".into()].to_vec())
        .include_state(true)
        .include_user_id(true)
        .execute()
        .await?;

    println!("All channels data: {:?}\n", channels_now);

    channels_now.iter().for_each(|channel| {
        println!("Channel: {}", channel.name);
        println!("Occupancy: {}", channel.occupancy);
        println!("Occupants: {:?}", channel.occupants);

        channel
            .occupants
            .iter()
            .for_each(|occupant| println!("Occupant: {:?}", occupant));

        println!();
    });

    Ok(())
}
