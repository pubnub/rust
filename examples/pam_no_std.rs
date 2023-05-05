#![no_std]

use hashbrown::HashMap;
use pubnub::core::PubNubError;
use pubnub::{access::*, Keyset, PubNubClientBuilder};

// As `no_std` does not support `Error` trait, we use `PubNubError` instead.
// In your program, you should handle the error properly for your use case.
fn main() -> Result<(), PubNubError> {
    // As `no_std` does not support `env::var`, you need to set the keys manually.
    let subscribe_key = "SDK_PAM_SUB_KEY";
    let publish_key = "SDK_PAM_PUB_KEY";
    let secret_key = "SDK_PAM_SEC_KEY";

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
        .meta(HashMap::from([("owner-role".into(), "admin".into())]))
        .execute_blocking()?;

    // here you can use the token to access PubNub API
    // ...

    // Revoke token permissions.
    client.revoke_token(grant_result.token).execute_blocking()?;

    // Handling API errors.
    let revoke_result = client.revoke_token("error_token").execute_blocking();
    if let Err(PubNubError::API { .. }) = revoke_result {
        // Handle API error.
    };

    Ok(())
}
