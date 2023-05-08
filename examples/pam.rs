use pubnub::{access::*, core::PubNubError, parse_token, Keyset, PubNubClientBuilder, Token};
use std::{collections::HashMap, env};

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
        .meta(HashMap::from([("owner-role".into(), "admin".into())]))
        .execute()
        .await?;

    println!(
        "Access token - call with metadata: {}",
        grant_result.token.clone()
    );

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
        .execute()
        .await?;

    println!(
        "Access token - call with various resources: {}",
        grant_result_various_resources.token.clone()
    );

    // Grant an authorized client multiple channels using RegEx
    let grant_result_multiple_channels_regex = client
        .grant_token(10)
        .authorized_user_id("my-authorized-user_id")
        .patterns(&[permissions::channel("^channel-[A-Za-z0-9]$").read()])
        .execute()
        .await?;

    println!(
        "Access token - call with multiple channels using RegEx: {}",
        grant_result_multiple_channels_regex.token.clone()
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
        .execute()
        .await?;

    println!(
        "Access token - call with different resources and patterns: {}",
        grant_result_different_levels_resources_patterns_regex
            .token
            .clone()
    );

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

    match parse_token(grant_result_various_resources.token.clone().as_str()) {
        Ok(Token::V2(token)) => println!("Token information: {token:#?}"),
        Err(err) => println!("Token parse error: {err:#?}"),
    }

    Ok(())
}
