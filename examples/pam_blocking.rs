use pubnub::core::PubNubError;
use pubnub::{access::*, Keyset, PubNubClientBuilder};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscribe_key = env::var("SDK_PAM_SUB_KEY")?;
    let publish_key = env::var("SDK_PAM_PUB_KEY")?;
    let secret_key = env::var("SDK_PAM_SEC_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_blocking_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: Some(secret_key),
        })
        .with_user_id("user_id")
        .build()?;

    // Grant permissions and generate access token.
    let grant_result = client
        .grant_token(10)
        .resources(&[
            permissions::channel_group("channel-group").read(),
            permissions::user_id("admin").update().delete(),
        ])
        .patterns(&[permissions::channel("^room-[a-zA-Z0-9]*$")
            .join()
            .read()
            .write()])
        .meta([("owner-role".into(), "admin".into())].into())
        .execute_blocking()?;

    println!("Access token - call with metadata: {}", grant_result.token);

    // Grant an authorized client different levels of access to various resources in a single call
    let grant_result_various_resources = client
        .grant_token(10)
        .authorized_user_id("my-authorized-user_id")
        .resources(&[
            permissions::channel("channel-a").read(),
            permissions::channel_group("channel-group-b").read(),
            permissions::user_id("uuid-c").get(),
            permissions::channel("channel-b").read().write(),
            permissions::channel("channel-c").read().write(),
            permissions::channel("channel-d").read().write(),
            permissions::user_id("uuid-d").get().update(),
        ])
        .execute_blocking()?;

    println!(
        "Access token - call with various resources: {}",
        grant_result_various_resources.token
    );

    // Grant an authorized client multiple channels using RegEx
    let grant_result_multiple_channels_regex = client
        .grant_token(10)
        .authorized_user_id("my-authorized-user_id")
        .patterns(&[permissions::channel("^channel-[A-Za-z0-9]$").read()])
        .execute_blocking()?;

    println!(
        "Access token - call with multiple channels using RegEx: {}",
        grant_result_multiple_channels_regex.token
    );

    // Grant an authorized client different levels of access to various resources and read access to channels using RegEx

    let grant_result_different_levels_resources_patterns_regex = client
        .grant_token(10)
        .authorized_user_id("my-authorized-user_id")
        .resources(&[
            permissions::channel("channel-a").read(),
            permissions::channel_group("channel-group-b").read(),
            permissions::user_id("uuid-c").get(),
            permissions::channel("channel-b").read().write(),
            permissions::channel("channel-c").read().write(),
            permissions::channel("channel-d").read().write(),
            permissions::user_id("uuid-d").get().update(),
        ])
        .patterns(&[permissions::channel("^channel-[A-Za-z0-9]$").read()])
        .execute_blocking()?;

    println!(
        "Access token - call with different resources and patterns: {}",
        grant_result_different_levels_resources_patterns_regex.token
    );

    // spawn a blocking thread to grant token
    let cloned = client.clone();
    let handle = std::thread::spawn(move || {
        // Grant token permissions.
        cloned
            .grant_token(10)
            .resources(&[
                permissions::channel_group("channel-group").read(),
                permissions::user_id("admin").update().delete(),
            ])
            .patterns(&[permissions::channel("^room-[a-zA-Z0-9]*$")
                .join()
                .read()
                .write()])
            .meta([("owner-role".into(), "admin".into())].into())
            .execute_blocking()
    });

    // Revoke token permissions.
    client.revoke_token(grant_result.token).execute_blocking()?;

    // Handling API errors.
    let revoke_result = client.revoke_token("error_token").execute_blocking();
    if let Err(PubNubError::API {
        status, message, ..
    }) = revoke_result
    {
        eprintln!("Expected error:\n {message} (HTTP status code: {status})");
    };

    // wait for the thread to finish
    let thread_grant_token_result = handle.join().expect("The revoke thread has panicked!")?;

    println!("Access token: {}", thread_grant_token_result.token);

    Ok(())
}
