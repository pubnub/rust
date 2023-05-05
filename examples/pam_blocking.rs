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

    println!("Access token: {}", grant_result.token);

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
