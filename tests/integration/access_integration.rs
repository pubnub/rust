#[cfg(test)]
mod integration {
    use pubnub::core::PubNubError;
    use pubnub::{access::*, Keyset, PubNubClientBuilder};
    use std::collections::HashMap;

    /// Keyset with default non-PAM enabled keys.
    fn default_keyset() -> Keyset<String> {
        Keyset {
            subscribe_key: option_env!("SDK_SUB_KEY").unwrap_or("demo").into(),
            publish_key: Some(option_env!("SDK_PUB_KEY").unwrap_or("demo").into()),
            secret_key: Some(option_env!("SDK_PUB_KEY").unwrap_or("demo").into()),
        }
    }

    /// Keyset with PAM enabled keys.
    fn pam_keyset() -> Keyset<String> {
        Keyset {
            subscribe_key: option_env!("SDK_PAM_SUB_KEY").unwrap_or("demo").into(),
            publish_key: Some(option_env!("SDK_PAM_PUB_KEY").unwrap_or("demo").into()),
            secret_key: Some(option_env!("SDK_PAM_SEC_KEY").unwrap_or("demo").into()),
        }
    }

    /// Grant token success
    #[tokio::test]
    async fn should_grant_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([
                ("string-val".into(), "hello there".into()),
                ("null-val".into(), ().into()),
            ]))
            .execute()
            .await;

        match result {
            Ok(result) => {
                assert!(!result.token.is_empty());
                Ok(())
            }
            Err(err) => {
                panic!("Unexpected error: {}", err);
            }
        }
    }

    /// Grant token failure because of signature mismatch.
    #[tokio::test]
    async fn should_not_grant_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(default_keyset())
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([("string-val".into(), "hello there".into())]))
            .execute()
            .await;

        match result {
            Ok(_) => panic!("Request should fail."),
            Err(err) => {
                if let PubNubError::API { status, .. } = err {
                    assert_eq!(status, 403);
                    Ok(())
                } else {
                    panic!("Unexpected error type.");
                }
            }
        }
    }

    /// Revoke token success
    #[tokio::test]
    async fn should_revoke_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([("string-val".into(), "hello there".into())]))
            .execute()
            .await?;

        let result = client.revoke_token(result.token).execute().await;

        if let Err(err) = result {
            eprintln!("Revoke token error: {err}");
            panic!("Request shouldn't fail");
        }

        Ok(())
    }

    /// Revoke token failure
    #[tokio::test]
    async fn should_not_revoke_expired_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("demo")
            .build()?;

        let result = client.revoke_token("some-token-here").execute().await;

        match result {
            Ok(_) => panic!("Request should fail."),
            Err(err) => {
                if let PubNubError::API { status, .. } = err {
                    // We are expecting failure because of wrong access token.
                    assert_eq!(status, 400);
                    Ok(())
                } else {
                    panic!("Unexpected error type.");
                }
            }
        }
    }

    /// Revoke token failure
    #[tokio::test]
    async fn should_not_revoke_malformed_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("demo")
            .build()?;

        let result = client.revoke_token("some-token-here").execute().await;

        match result {
            Ok(_) => panic!("Request should fail."),
            Err(err) => {
                if let PubNubError::API { status, .. } = err {
                    // We are expecting failure because of wrong access token.
                    assert_eq!(status, 400);
                    Ok(())
                } else {
                    panic!("Unexpected error type.");
                }
            }
        }
    }
}
