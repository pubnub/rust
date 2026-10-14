#[cfg(test)]
mod integration {
    use pubnub::core::PubNubError;
    use pubnub::dx::parse_token::MetaValue;
    use pubnub::{access::*, parse_token, Keyset, PubNubClientBuilder, Token};
    use std::collections::HashMap;

    /// Read required keyset value from the environment.
    ///
    /// Values are provided by CI secrets. Nothing is hardcoded.
    fn required_env(name: &str) -> String {
        std::env::var(name)
            .unwrap_or_else(|_| panic!("{name} must be set to run PAM integration tests"))
    }

    /// Keyset with PAM enabled keys.
    fn pam_keyset() -> Keyset<String> {
        Keyset {
            subscribe_key: required_env("SDK_PAM_SUB_KEY"),
            publish_key: Some(required_env("SDK_PAM_PUB_KEY")),
            secret_key: Some(required_env("SDK_PAM_SEC_KEY")),
        }
    }

    /// Keyset with PAM enabled subscribe / publish keys and wrong secret key.
    fn pam_keyset_with_wrong_secret() -> Keyset<String> {
        Keyset {
            subscribe_key: required_env("SDK_PAM_SUB_KEY"),
            publish_key: Some(required_env("SDK_PAM_PUB_KEY")),
            secret_key: Some("sec-c-invalid".into()),
        }
    }

    /// Grant token with resources and meta, then parse it back.
    #[tokio::test]
    async fn should_grant_token_and_parse_resources() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let granted = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([
                ("string-val".into(), "hello there".into()),
                ("null-val".into(), ().into()),
            ]))
            .execute()
            .await?;
        assert!(!granted.token.is_empty());

        let Token::V2(token) = parse_token(&granted.token)?;
        assert_eq!(token.version, 2);
        assert_eq!(token.ttl, 10);

        let resources = token.resources.expect("resources should be present");
        let channel = resources
            .channels
            .get("test-channel")
            .expect("test-channel should be granted");
        assert!(channel.read);
        assert!(channel.write);
        assert!(!channel.manage);

        assert!(token.categories.is_none());
        assert!(matches!(
            token.meta.get("string-val"),
            Some(MetaValue::String(value)) if value == "hello there"
        ));

        Ok(())
    }

    /// Grant token with categories only, then parse it back.
    #[tokio::test]
    async fn should_grant_categories_only_token_and_parse_it(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let categories: Vec<Box<dyn permissions::Permission>> = vec![
            permissions::channels_category().get(),
            permissions::uuids_category().get(),
        ];
        let granted = client
            .grant_token(10)
            .categories(&categories)
            .execute()
            .await?;

        let Token::V2(token) = parse_token(&granted.token)?;
        let categories = token.categories.expect("categories should be present");
        assert!(categories.channels.expect("channels category").get);
        assert!(categories.uuids.expect("uuids category").get);

        assert!(token.resources.is_none_or(|r| r.channels.is_empty()));
        assert!(token.patterns.is_none_or(|p| p.channels.is_empty()));

        Ok(())
    }

    /// Grant token with resources, patterns and categories, then parse it back.
    #[tokio::test]
    async fn should_grant_all_permission_kinds_and_parse_them(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let categories: Vec<Box<dyn permissions::Permission>> =
            vec![permissions::uuids_category().get()];
        let granted = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read()])
            .patterns(&[permissions::channel("test-*").write()])
            .categories(&categories)
            .execute()
            .await?;

        let Token::V2(token) = parse_token(&granted.token)?;

        let resources = token.resources.expect("resources should be present");
        assert!(resources.channels["test-channel"].read);

        let patterns = token.patterns.expect("patterns should be present");
        assert!(patterns.channels["test-*"].write);

        let categories = token.categories.expect("categories should be present");
        assert!(categories.channels.is_none());
        assert!(categories.uuids.expect("uuids category").get);

        Ok(())
    }

    /// Grant token failure because of signature mismatch.
    #[tokio::test]
    async fn should_not_grant_token_with_wrong_secret() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset_with_wrong_secret())
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .execute()
            .await;

        match result {
            Ok(_) => panic!("Request should fail."),
            Err(PubNubError::API { status, .. }) => {
                assert_eq!(status, 403);
                Ok(())
            }
            Err(_) => panic!("Unexpected error type."),
        }
    }

    /// Grant token then revoke it.
    #[tokio::test]
    async fn should_revoke_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("test")
            .build()?;

        let granted = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .execute()
            .await?;

        client.revoke_token(granted.token).execute().await?;

        Ok(())
    }

    /// Revoke token failure with expired / unknown token.
    #[tokio::test]
    async fn should_not_revoke_expired_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(pam_keyset())
            .with_user_id("demo")
            .build()?;

        let result = client.revoke_token("some-token-here").execute().await;

        match result {
            Ok(_) => panic!("Request should fail."),
            Err(PubNubError::API { status, .. }) => {
                // We are expecting failure because of wrong access token.
                assert_eq!(status, 400);
                Ok(())
            }
            Err(_) => panic!("Unexpected error type."),
        }
    }

    /// Parse failure for token which is not base64 / CBOR.
    #[test]
    fn should_not_parse_malformed_token() {
        assert!(parse_token("some-token-here").is_err());
    }
}
