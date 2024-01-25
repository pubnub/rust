//! PAM grant token module.
//!
//! This module contains `Grant Token` request builder.

use crate::{
    core::{
        error::PubNubError,
        utils::headers::{APPLICATION_JSON, CONTENT_TYPE},
        Deserializer, Serializer, Transport, TransportMethod, TransportRequest,
    },
    dx::{access::*, pubnub_client::PubNubClientInstance},
    lib::{
        alloc::{boxed::Box, format, string::ToString, vec},
        collections::HashMap,
    },
};
use derive_builder::Builder;

/// The [`GrantTokenRequestBuilder`] is used to build grant access token
/// permissions to access specific resource endpoints request that is sent to
/// the [`PubNub`] network.
///
/// This struct used by the [`grant_token`] method of the [`PubNubClient`].
/// The [`grant_token`] method is used to generate access token.
///
/// [`PubNub`]:https://www.pubnub.com/
/// [`PubNubClient`]: crate::PubNubClient
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::access)", validate = "Self::validate"),
    no_std
)]
pub struct GrantTokenRequest<'pa, T, S, D>
where
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
{
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) pubnub_client: PubNubClientInstance<T, D>,

    /// Request payload serializer.
    #[builder(field(vis = "pub(in crate::dx::access)"), setter(custom))]
    pub(in crate::dx::access) serializer: S,

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

    /// Extra metadata to be included into resulting access token. Values must
    /// be scalar only.
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
pub struct GrantTokenRequestWithSerializerBuilder<T, D> {
    /// Current client which can provide transportation to perform the request.
    pub(in crate::dx::access) pubnub_client: PubNubClientInstance<T, D>,

    /// How long (in minutes) the generated token should be valid.
    pub(in crate::dx::access) ttl: usize,
}

impl<'pa, T, S, D> GrantTokenRequest<'pa, T, S, D>
where
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
{
    /// Create transport request from the request builder.
    pub(in crate::dx::access) fn transport_request(&self) -> TransportRequest {
        let config = &self.pubnub_client.config;
        let payload = GrantTokenPayload::new(self);
        let body = self.serializer.serialize(&payload).unwrap_or(vec![]);

        TransportRequest {
            path: format!("/v3/pam/{}/grant", &config.subscribe_key),
            query_parameters: Default::default(),
            method: TransportMethod::Post,
            headers: [(CONTENT_TYPE.to_string(), APPLICATION_JSON.to_string())].into(),
            body: if !body.is_empty() { Some(body) } else { None },
            #[cfg(feature = "std")]
            timeout: config.transport.request_timeout,
        }
    }
}

impl<'pa, T, S, D> GrantTokenRequestBuilder<'pa, T, S, D>
where
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
{
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        builders::validate_configuration(&self.pubnub_client)
    }
}

impl<'pa, T, S, D> GrantTokenRequestBuilder<'pa, T, S, D>
where
    T: Transport + 'static,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<GrantTokenResult, PubNubError> {
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<GrantTokenResponseBody, _, _, _>(
                &client.transport,
                deserializer,
                #[cfg(feature = "std")]
                &client.config.transport.retry_configuration,
                #[cfg(feature = "std")]
                &client.runtime,
            )
            .await
    }
}

#[cfg(feature = "blocking")]
impl<'pa, T, S, D> GrantTokenRequestBuilder<'pa, T, S, D>
where
    T: crate::core::blocking::Transport,
    S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    D: Deserializer + 'static,
{
    /// Execute synchronous request and return the result.
    ///
    /// This method is synchronous and will return result which will resolve to
    /// a [`RevokeTokenResult`] or [`PubNubError`].
    ///
    /// # Example
    /// ```no_run
    /// # use pubnub::{PubNubClientBuilder, Keyset, access::*};
    /// # use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #     PubNubClientBuilder::with_reqwest_blocking_transport()
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
    ///          ("role".to_string(), "administrator".into()),
    ///          ("access-duration".to_string(), 2800.into()),
    ///          ("ping-interval".to_string(), 1754.88.into()),
    ///      ]))
    ///     .execute_blocking()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn execute_blocking(self) -> Result<GrantTokenResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<GrantTokenResponseBody, _, _, _>(&client.transport, deserializer)
    }
}

#[cfg(not(feature = "serde"))]
impl<T, D> GrantTokenRequestWithSerializerBuilder<T, D> {
    /// Add custom serializer.
    ///
    /// Adds the serializer to the [`GrantTokenRequestBuilder`].
    ///
    /// Instance of [`GrantTokenRequestWithDeserializerBuilder`] returned.
    pub fn serialize_with<'request, S>(
        self,
        serializer: S,
    ) -> GrantTokenRequestBuilder<'request, T, S, D>
    where
        S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
    {
        GrantTokenRequestBuilder {
            pubnub_client: Some(self.pubnub_client),
            ttl: Some(self.ttl),
            serializer: Some(serializer),
            ..Default::default()
        }
    }
}
