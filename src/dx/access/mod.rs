//! Access Manager module.
//!
//! Manage resource endpoints access token.
//! This access manager module contains the [`GrantTokenRequestBuilder`] and
//! [`Permission`].

#[doc(inline)]
pub(crate) use payloads::*;
pub(crate) mod payloads;

#[doc(inline)]
pub use types::MetaValue;
pub mod types;

#[doc(inline)]
pub use result::{
    GrantTokenResponseBody, GrantTokenResult, RevokeTokenResponseBody, RevokeTokenResult,
};
pub mod result;

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
pub use permissions::*;
pub mod permissions;

#[cfg(feature = "serde")]
use crate::providers::{
    deserialization_serde::SerdeDeserializer, serialization_serde::SerdeSerializer,
};
use crate::{core::Transport, dx::PubNubClient};

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// Create grant token permissions request builder.
    /// This method is used to generate token with required permissions.
    ///
    /// Instance of [`GrantTokenRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    /// #    PubNubClientBuilder, Keyset,
    /// };
    /// # use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .grant_token(10)
    ///     .resources(&[permissions::channel("test-channel").read().write()])
    ///     .meta(HashMap::from([
    ///          ("string-val".into(), MetaValue::String("hello there".into())),
    ///          ("null-val".into(), MetaValue::Null),
    ///      ]))
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(feature = "serde")]
    pub fn grant_token(
        &self,
        ttl: usize,
    ) -> GrantTokenRequestBuilder<T, SerdeSerializer, SerdeDeserializer> {
        GrantTokenRequestBuilder {
            pubnub_client: Some(self.clone()),
            serializer: Some(SerdeSerializer),
            deserializer: Some(SerdeDeserializer),
            ttl: Some(ttl),
            ..Default::default()
        }
    }
    /// Create grant token permissions request builder.
    /// This method is used to generate token with required permissions.
    ///
    /// Instance of [`GrantTokenRequestWithSerializerBuilder`] returned.
    ///
    /// # Example
    /// ```rust, no_run
    /// use pubnub::{
    ///     access::*,
    ///     core::{Deserializer, PubNubError, Serializer},
    /// #    PubNubClientBuilder, Keyset,
    /// };
    /// # use std::collections::HashMap;
    ///
    /// struct MySerializer;
    /// struct MyDeserializer;
    ///
    /// impl<'se> Serializer<'se, GrantTokenPayload> for MySerializer {
    ///    fn serialize(&self, object: &'se T) -> Result<Vec<u8>, PubNubError> {
    ///         // ...
    /// #        Ok(vec![])
    ///     }
    /// }
    ///
    /// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
    ///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
    ///         // ...
    /// #        Ok(GrantTokenResult { token: "<generated token>".into() })
    ///     }
    /// }
    ///
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .grant_token(10)
    ///     .serialize_with(MySerializer)
    ///     .derialize_with(MyDeserializer)
    ///     .resources(&[permissions::channel("test-channel").read().write()])
    ///     .meta(HashMap::from([
    ///          ("string-val".into(), MetaValue::String("hello there".into())),
    ///          ("null-val".into(), MetaValue::Null),
    ///      ]))
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "serde"))]
    pub fn grant_token(&self, ttl: usize) -> GrantTokenRequestWithSerializerBuilder<T> {
        GrantTokenRequestWithSerializerBuilder {
            pubnub_client: self.clone(),
            ttl,
        }
    }

    /// Create grant token request builder.
    /// This method is used to revoke token permissions.
    ///
    /// Instance of [`RevokeTokenRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    /// #    PubNubClientBuilder, Keyset,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .revoke_token("p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI")
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(feature = "serde")]
    pub fn revoke_token<S>(&self, token: S) -> RevokeTokenRequestBuilder<T, SerdeDeserializer>
    where
        S: Into<String>,
    {
        RevokeTokenRequestBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(SerdeDeserializer),
            token: Some(token.into()),
        }
    }

    /// Create revoke token permissions request builder.
    /// This method is used to revoke token permissions.
    ///
    /// Instance of [`RevokeTokenRequestWithDeserializerBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// use pubnub::{
    ///     access::*,
    ///     core::{Deserializer, PubNubError, Serializer},
    /// #    PubNubClientBuilder, Keyset,
    /// };
    ///
    /// struct MyDeserializer;
    ///
    /// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
    ///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
    ///         // ...
    /// #        Ok(GrantTokenResult)
    ///     }
    /// }
    ///
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_transport()
    /// #         .with_keyset(Keyset {
    /// #              subscribe_key: "demo",
    /// #              publish_key: Some("demo"),
    /// #              secret_key: Some("demo")
    /// #          })
    /// #         .with_user_id("uuid")
    /// #         .build()?;
    /// pubnub
    ///     .revoke_token("p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI".into())
    ///     .derialize_with(MyDeserializer)
    ///     .execute()
    ///     .await?;
    /// #     Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "serde"))]
    pub fn revoke_token<S>(&self, token: S) -> RevokeTokenRequestWithDeserializerBuilder<T>
    where
        S: Into<String>,
    {
        RevokeTokenRequestWithDeserializerBuilder {
            pubnub_client: self.clone(),
            token: token.into(),
        }
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::core::PubNubError;
    use crate::{dx::access::types::MetaValue, Keyset, PubNubClientBuilder};
    use std::collections::HashMap;

    /// Grant token success
    #[tokio::test]
    async fn should_grant_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(Keyset {
                subscribe_key: option_env!("SDK_PAM_SUB_KEY").unwrap_or("demo"),
                publish_key: Some(option_env!("SDK_PAM_PUB_KEY").unwrap_or("demo")),
                secret_key: Some(option_env!("SDK_PAM_SEC_KEY").unwrap_or("demo")),
            })
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([
                ("string-val".into(), MetaValue::String("hello there".into())),
                ("null-val".into(), MetaValue::Null),
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
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: Some("demo"),
            })
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([(
                "string-val".into(),
                MetaValue::String("hello there".into()),
            )]))
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
            .with_keyset(Keyset {
                subscribe_key: option_env!("SDK_PAM_SUB_KEY").unwrap_or("demo"),
                publish_key: Some(option_env!("SDK_PAM_PUB_KEY").unwrap_or("demo")),
                secret_key: Some(option_env!("SDK_PAM_SEC_KEY").unwrap_or("demo")),
            })
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([(
                "string-val".into(),
                MetaValue::String("hello there".into()),
            )]))
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
            .with_keyset(Keyset {
                subscribe_key: option_env!("SDK_PAM_SUB_KEY").unwrap_or("demo"),
                publish_key: Some(option_env!("SDK_PAM_PUB_KEY").unwrap_or("demo")),
                secret_key: Some(option_env!("SDK_PAM_SEC_KEY").unwrap_or("demo")),
            })
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
            .with_keyset(Keyset {
                subscribe_key: option_env!("SDK_PAM_SUB_KEY").unwrap_or("demo"),
                publish_key: Some(option_env!("SDK_PAM_PUB_KEY").unwrap_or("demo")),
                secret_key: Some(option_env!("SDK_PAM_SEC_KEY").unwrap_or("demo")),
            })
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
