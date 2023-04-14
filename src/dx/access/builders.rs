//! Access manager builders module.
//!
//! This module contains all builders for the PAM management operations.
//!

use crate::{
    core::{
        headers::{APPLICATION_JSON, CONTENT_TYPE},
        Deserializer, PubNubError, Serializer, Transport, TransportMethod, TransportRequest,
    },
    dx::access::{
        payloads::GrantTokenPayload, permissions, types::MetaValue, GrantTokenResponseBody,
        GrantTokenResult,
    },
    PubNubClient,
};
use derive_builder::Builder;
use std::collections::HashMap;

/// The [`AccessManagerGrantTokenBuilder`] is used to generate access token to
/// access specific resource endpoints.
///
/// This struct used by the [`grant_token_with_ttl`] method of the
/// [`PubNubClient`].
/// The [`grant_token_with_ttl`] method is used to generate access token.
/// [`AccessManagerGrantTokenBuilder`] is used to build the request that is sent
/// to the [`PubNub`] network.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(super)", validate = "Self::validate")
)]
pub struct GrantTokenRequest<'pa, T, S, D>
where
    T: Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    D: for<'dl> Deserializer<'dl, GrantTokenResponseBody>,
{
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) pubnub_client: PubNubClient<T>,

    /// How long (in minutes) the generated token should be valid.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) ttl: usize,

    /// Request payload serializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) serializer: S,

    /// Service response deserializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(super) deserializer: D,

    /// A user ID, which is authorized to use the token to make API requests to
    /// PubNub.
    #[builder(
        setter(into, strip_option),
        default = "None",
        field(vis = "pub(in crate::dx::access)")
    )]
    pub(super) authorized_user_id: Option<String>,

    /// Extra metadata to be published with the request. Values must be scalar
    /// only.
    #[builder(
        setter(strip_option),
        default = "None",
        field(vis = "pub(in crate::dx::access)")
    )]
    pub(super) meta: Option<HashMap<String, MetaValue>>,

    /// List of permissions mapped to resource identifiers.
    #[builder(
        setter(strip_option),
        default = "None",
        field(vis = "pub(in crate::dx::access)")
    )]
    pub(super) resources: Option<&'pa [Box<dyn permissions::Permission>]>,

    /// List of permissions mapped to RegExp match expressions.
    #[builder(
        setter(strip_option),
        default = "None",
        field(vis = "pub(in crate::dx::access)")
    )]
    pub(super) patterns: Option<&'pa [Box<dyn permissions::Permission>]>,
}

/// The [`GrantTokenRequestWithSerializerBuilder`] adds the serializer to the
/// [`GrantTokenRequestBuilder`]
///
/// Create new PAMv3 grant token builder.
/// This method is used to generate token with required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::{PubNubClientBuilder, Keyset};
/// use pubnub::{
///     dx::access::{permissions, permissions::*, types::MetaValue},
///     core::{Serializer, Deserializer, PubNubError}
/// };
/// # use std::collections::HashMap;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// struct MySerializer;
/// struct MyDeserializer;
///
/// impl<'de> Serializer<'se, GrantTokenPayload> for MySerializer {
///     fn serialize(&self, response: &'de [u8]) -> Result<Vec<u8>, PubNubError> {
///         // ...
///         # Ok(Vec<u8>)
///     }
/// }
///
/// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
///         // ...
///         # Ok(GrantTokenResult)
///     }
/// }
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
pub struct GrantTokenRequestWithSerializerBuilder<T>
where
    T: Transport,
{
    /// Current client which can provide transportation to perform the request.
    pubnub_client: PubNubClient<T>,

    /// How long (in minutes) the generated token should be valid.
    ttl: usize,
}

/// The [`GrantTokenRequestWithDeserializerBuilder`] adds the deserializer to
/// the [`GrantTokenRequestBuilder`]
///
/// Create new PAMv3 grant token builder.
/// This method is used to generate token with required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::{PubNubClientBuilder, Keyset};
/// use pubnub::{
///     dx::access::{permissions, permissions::*, types::MetaValue},
///     core::{Serializer, Deserializer, PubNubError}
/// };
/// # use std::collections::HashMap;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// struct MySerializer;
/// struct MyDeserializer;
///
/// impl<'de> Serializer<'se, GrantTokenPayload> for MySerializer {
///     fn serialize(&self, response: &'de [u8]) -> Result<Vec<u8>, PubNubError> {
///         // ...
///         # Ok(Vec<u8>)
///     }
/// }
///
/// impl<'de> Deserializer<'de, GrantTokenResponseBody> for MyDeserializer {
///     fn deserialize(&self, response: &'de [u8]) -> Result<GrantTokenResult, PubNubError> {
///         // ...
///         # Ok(GrantTokenResult)
///     }
/// }
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
pub struct GrantTokenRequestWithDeserializerBuilder<T, S>
where
    T: Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
{
    /// Current client which can provide transportation to perform the request.
    pubnub_client: PubNubClient<T>,

    /// How long (in minutes) the generated token should be valid.
    ttl: usize,

    /// Request payload serializer.
    serializer: Option<S>,
}

impl<'pa, T, S, D> GrantTokenRequest<'pa, T, S, D>
where
    T: Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    D: for<'ds> Deserializer<'ds, GrantTokenResponseBody>,
{
    /// Create transport request from the requqest builder.
    fn transport_request(&self) -> TransportRequest {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let payload = GrantTokenPayload::new(self);
        let body = self.serializer.serialize(&payload).unwrap_or(vec![]);

        TransportRequest {
            path: format!("/v3/pam/{}/grant", sub_key),
            query_parameters: Default::default(),
            method: TransportMethod::Post,
            headers: [(CONTENT_TYPE.into(), APPLICATION_JSON.into())].into(),
            body: if !body.is_empty() { Some(body) } else { None },
        }
    }
}

#[cfg(not(feature = "serde"))]
impl<T> GrantTokenRequestWithSerializerBuilder<T>
where
    T: Transport,
{
    /// Add custom serializer.
    ///
    /// Adds the serializer to the [`GrantTokenRequestBuilder`].
    ///
    /// Instance of [`GrantTokenRequestWithDeserializerBuilder`] returned.
    pub fn serialize_with<D, S>(
        self,
        serializer: S,
    ) -> GrantTokenRequestWithDeserializerBuilder<T, S>
    where
        D: for<'ds> Deserializer<'ds, GrantTokenResponseBody>,
        S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    {
        GrantTokenRequestWithDeserializerBuilder {
            pubnub_client: self.pubnub_client,
            ttl: self.ttl,
            serializer: Some(serializer),
        }
    }
}

#[cfg(not(feature = "serde"))]
impl<T, S> GrantTokenRequestWithDeserializerBuilder<T, S>
where
    T: Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
{
    /// Add custom deserializer.
    ///
    /// Adds the deserializer to the [`GrantTokenRequestBuilder`].
    ///
    /// Instance of [`GrantTokenRequestBuilder`] returned.
    pub fn deserialize_with<'builder, D>(
        self,
        deserializer: D,
    ) -> GrantTokenRequestBuilder<'builder, T, S, D>
    where
        D: for<'ds> Deserializer<'ds, GrantTokenResponseBody>,
    {
        GrantTokenRequestBuilder {
            pubnub_client: Some(self.pubnub_client),
            ttl: Some(self.ttl),
            serializer: self.serializer,
            deserializer: Some(deserializer),
            ..Default::default()
        }
    }
}

impl<'pa, T, S, D> GrantTokenRequestBuilder<'pa, T, S, D>
where
    T: Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    D: for<'ds> Deserializer<'ds, GrantTokenResponseBody>,
{
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let mut perm_len = 0;
        if let Some(resources) = self.resources {
            resources
                .unwrap_or(&[])
                .iter()
                .for_each(|perm| perm_len += *perm.value());
        }
        if let Some(patterns) = self.patterns {
            patterns
                .unwrap_or(&[])
                .iter()
                .for_each(|pat| perm_len += *pat.value());
        }

        if perm_len == 0 {
            return Err(
                "The list of resources and patterns permissions is empty or doesn't have any \
                permission associated with them."
                    .into(),
            );
        }

        Ok(())
    }

    /// Build and call request.
    pub async fn execute(self) -> Result<GrantTokenResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self.build().map_err(|err| match err {
            GrantTokenRequestBuilderError::UninitializedField(msg) => {
                PubNubError::general_api_error(msg, None)
            }
            GrantTokenRequestBuilderError::ValidationError(msg) => {
                PubNubError::general_api_error(msg, None)
            }
        })?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = request.deserializer;
        client
            .transport
            .send(transport_request)
            .await?
            .body
            .map(|bytes| deserializer.deserialize(&bytes))
            .map_or(
                Err(PubNubError::general_api_error(
                    "No body in the response!",
                    None,
                )),
                |response_body| {
                    response_body.and_then::<GrantTokenResult, _>(|body| body.try_into())
                },
            )
    }
}
