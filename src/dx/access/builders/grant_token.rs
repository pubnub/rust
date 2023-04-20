//! PAM grant token module.
//!
//! This module contains `Grant Token` request builder.

use crate::{
    core::{
        error::PubNubError,
        headers::{APPLICATION_JSON, CONTENT_TYPE},
        Deserializer, Serializer, Transport, TransportMethod, TransportRequest,
    },
    dx::{access::*, PubNubClient},
};
use derive_builder::Builder;
use std::collections::HashMap;

/// The [`GrantTokenRequestBuilder`] is used to build grant access token
/// permissions to access specific resource endpoints request that is sent to
/// the [`PubNub`] network.
///
/// This struct used by the [`grant_token`] method of the [`PubNubClient`].
/// The [`grant_token`] method is used to generate access token.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::access)", validate = "Self::validate")
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

    /// Request payload serializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) serializer: S,

    /// Service response deserializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) deserializer: D,

    /// How long (in minutes) the generated token should be valid.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) ttl: usize,

    /// A user ID, which is authorized to use the token to make API requests to
    /// PubNub.
    #[builder(
        field(vis = "pub(in crate::dx::access)"),
        setter(into, strip_option),
        default = "None"
    )]
    pub authorized_user_id: Option<String>,

    /// Extra metadata to be published with the request. Values must be scalar
    /// only.
    #[builder(
        field(vis = "pub(in crate::dx::access)"),
        setter(strip_option),
        default = "None"
    )]
    pub meta: Option<HashMap<String, MetaValue>>,

    /// List of permissions mapped to resource identifiers.
    #[builder(
        field(vis = "pub(in crate::dx::access)"),
        setter(strip_option),
        default = "None"
    )]
    pub resources: Option<&'pa [Box<dyn permissions::Permission>]>,

    /// List of permissions mapped to RegExp match expressions.
    #[builder(
        field(vis = "pub(in crate::dx::access)"),
        setter(strip_option),
        default = "None"
    )]
    pub patterns: Option<&'pa [Box<dyn permissions::Permission>]>,
}

/// The [`GrantTokenRequestWithSerializerBuilder`] is used to build grant access
/// token permissions to access specific resource endpoints request that is sent
/// to the [`PubNub`] network.
///
/// This struct used by the [`grant_token`] method of the [`PubNubClient`] and
/// let specify custom serializer for payload sent to [`PubNub`] network.
/// The [`grant_token`] method is used to generate access token.
///
/// [`PubNub`]:https://www.pubnub.com/
#[cfg(not(feature = "serde"))]
pub struct GrantTokenRequestWithSerializerBuilder<T>
where
    T: Transport,
{
    /// Current client which can provide transportation to perform the request.
    pub(in crate::dx::access) pubnub_client: PubNubClient<T>,

    /// How long (in minutes) the generated token should be valid.
    pub(in crate::dx::access) ttl: usize,
}

/// The [`GrantTokenRequestWithDeserializerBuilder`] is used to build grant access
/// token permissions to access specific resource endpoints request that is sent
/// to the [`PubNub`] network.
///
/// This struct used by the [`grant_token`] method of the [`PubNubClient`] and
/// let specify custom deserializer for [`PubNub`] network response.
/// The [`grant_token`] method is used to generate access token.
///
/// [`PubNub`]:https://www.pubnub.com/
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
    /// Create transport request from the request builder.
    pub(in crate::dx::access) fn transport_request(&self) -> TransportRequest {
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
        let mut perm_len: u64 = 0;
        if let Some(resources) = self.resources {
            resources
                .unwrap_or(&[])
                .iter()
                .for_each(|perm| perm_len += *perm.value() as u64);
        }

        if let Some(patterns) = self.patterns {
            patterns
                .unwrap_or(&[])
                .iter()
                .for_each(|perm| perm_len += *perm.value() as u64);
        }

        if perm_len == 0 {
            return Err(
                "The list of resources and patterns permissions is empty or doesn't have any \
                permission associated with them."
                    .into(),
            );
        }

        builders::validate_configuration(&self.pubnub_client)
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
