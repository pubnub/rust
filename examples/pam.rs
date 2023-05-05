use pubnub::core::PubNubError;
use pubnub::{access::*, Keyset, PubNubClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscribe_key = env::var("SDK_PAM_SUB_KEY")?;
    let publish_key = env::var("SDK_PAM_PUB_KEY")?;
    let secret_key = env::var("SDK_PAM_SEC_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_transport()
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
        .execute()
        .await?;

    println!("Access token: {}", grant_result.token.clone());

    // Revoke token permissions.
    client
        .revoke_token(grant_result.token.clone())
        .execute()
        .await?;

    // Handling API errors.
    let revoke_result = client.revoke_token("error_token").execute().await;
    if let Err(PubNubError::API {
        status, message, ..
    }) = revoke_result
    {
        eprintln!(
            "Expected error:\n {} (HTTP status code: {})",
            message, status
        );
    };

    Ok(())
}
