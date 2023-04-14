//! Access Manager module.
//!
//! Manage resource endpoints access token.
//! This access manager module contains the [`GrantTokenRequestBuilder`] and
//! [`Permission`].

pub mod payloads;
pub mod types;

#[doc(inline)]
pub(crate) use result::GrantTokenResponseBody;
#[doc(inline)]
pub use result::GrantTokenResult;
pub mod result;

#[doc(inline)]
pub use builders::{GrantTokenRequest, GrantTokenRequestBuilder};
pub mod builders;

pub use permissions::*;
pub mod permissions;
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;
#[cfg(feature = "serde")]
use crate::providers::serialization_serde::SerdeSerializer;
use crate::{core::Transport, dx::PubNubClient};

impl<T> PubNubClient<T>
where
    T: Transport,
{
    /// Create new PAMv3 grant token builder.
    /// This method is used to generate token with required permissions.
    ///
    /// Instance of [`GrantTokenRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use pubnub::{PubNubClientBuilder, Keyset};
    /// use pubnub::dx::access::{permissions, permissions::*, types::MetaValue};
    /// # use std::collections::HashMap;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
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
    ///     .grant_token_with_ttl(10)
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
    pub fn grant_token_with_ttl(
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
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{dx::access::types::MetaValue, Keyset, PubNubClientBuilder};
    use std::collections::HashMap;

    #[tokio::test]
    async fn should_grant_token() -> Result<(), Box<dyn std::error::Error>> {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: Some("demo"),
            })
            .with_user_id("test")
            .build()?;

        let result = client
            .grant_token_with_ttl(10)
            .resources(&[permissions::channel("test-channel").read().write()])
            .meta(HashMap::from([
                ("string-val".into(), MetaValue::String("hello there".into())),
                ("null-val".into(), MetaValue::Null),
            ]))
            .execute()
            .await;
        assert!(result.is_err());

        Ok(())
    }
}
