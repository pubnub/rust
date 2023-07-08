//! # PubNub subscribe module.
//!
//! This module has all the builders for subscription to real-time updates from
//! a list of channels and channel groups.

use crate::{
    core::{
        utils::encoding::join_url_encoded,
        Deserializer, PubNubError, Transport, {TransportMethod, TransportRequest},
    },
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{
            builders,
            cancel::CancellationTask,
            result::{SubscribeResponseBody, SubscribeResult},
            SubscribeCursor,
        },
    },
    lib::{
        alloc::{
            format,
            string::{String, ToString},
            vec::Vec,
        },
        collections::HashMap,
    },
};
use derive_builder::Builder;
use futures::{select, Future, FutureExt};

/// The [`SubscribeRequestBuilder`] is used to build subscribe request which
/// will be used for real-time updates notification from the [`PubNub`] network.
///
/// This struct used by the [`subscribe`] method of the [`PubNubClient`].
/// The [`subscribe`] method is used to subscribe and receive real-time updates
/// from the [`PubNub`] network.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::subscribe)", validate = "Self::validate"),
    no_std
)]
#[allow(dead_code, missing_docs)]
pub struct SubscribeRequest<T, D>
where
    T: Transport + Send,
    D: for<'ds> Deserializer<'ds, SubscribeResponseBody> + Send,
{
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) pubnub_client: PubNubClientInstance<T>,

    /// Service response deserializer.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) deserializer: D,

    /// Channels from which real-time updates should be received.
    ///
    /// List of channels on which [`PubNubClient`] will subscribe and notify
    /// about received real-time updates.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), default = "Vec::new()")]
    pub(in crate::dx::subscribe) channels: Vec<String>,

    /// Channel groups from which real-time updates should be received.
    ///
    /// List of groups of channels on which [`PubNubClient`] will subscribe and
    /// notify about received real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Vec::new()"
    )]
    pub(in crate::dx::subscribe) channel_groups: Vec<String>,

    /// Time cursor.
    ///
    /// Cursor used by subscription loop to identify point in time after
    /// which updates will be delivered.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Default::default()"
    )]
    pub(in crate::dx::subscribe) cursor: SubscribeCursor,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(skip),
        default = "300"
    )]
    pub(in crate::dx::subscribe) heartbeat: u32,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,
}

/// The [`SubscribeRequestWithDeserializerBuilder`] is used to build subscribe
/// request which will be used for real-time updates notification from the
/// [`PubNub`] network.
///
/// This struct used by the [`subscribe`] method of the [`PubNubClient`] and let
/// specify custom deserializer for received real-time updates from
/// the [`PubNub`] network.
/// The [`subscribe`] method is used to subscribe and receive real-time updates
/// from the [`PubNub`] network.
///
/// [`PubNub`]:https://www.pubnub.com/
#[cfg(not(feature = "serde"))]
pub struct SubscribeRequestWithDeserializerBuilder<T> {
    /// Current client which can provide transportation to perform the request.
    pub(in crate::dx::subscribe) pubnub_client: PubNubClientInstance<T>,
}

impl<T, D> SubscribeRequest<T, D>
where
    T: Transport,
    D: for<'ds> Deserializer<'ds, SubscribeResponseBody>,
{
    /// Create transport request from the request builder.
    pub(in crate::dx::subscribe) fn transport_request(&self) -> TransportRequest {
        let sub_key = &self.pubnub_client.config.subscribe_key;
        let channels = join_url_encoded(
            self.channels
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            ",",
        )
        .unwrap_or(",".into());
        let mut query: HashMap<String, String> = HashMap::new();
        query.extend::<HashMap<String, String>>(self.cursor.clone().into());

        // Serialize list of channel groups and add into query parameters list.
        join_url_encoded(
            self.channel_groups
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            ",",
        )
        .filter(|string| !string.is_empty())
        .and_then(|channel_groups| query.insert("channel-group".into(), channel_groups));

        self.filter_expression
            .as_ref()
            .filter(|e| !e.is_empty())
            .and_then(|e| query.insert("filter-expr".into(), e.into()));

        TransportRequest {
            path: format!("/v2/subscribe/{sub_key}/{channels}/0"),
            query_parameters: query,
            method: TransportMethod::Get,
            ..Default::default()
        }
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: Transport,
    D: for<'ds> Deserializer<'ds, SubscribeResponseBody>,
{
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        builders::validate_configuration(&self.pubnub_client).and_then(|_| {
            if channels_len == groups_len && channels_len == 0 {
                Err("Either channels or channel groups should be provided".into())
            } else {
                Ok(())
            }
        })
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: Transport,
    D: for<'ds> Deserializer<'ds, SubscribeResponseBody>,
{
    /// Build and call request.
    pub fn execute(
        self,
        cancel_task: CancellationTask,
    ) -> impl Future<Output = Result<SubscribeResult, PubNubError>> {
        async move {
            // Build request instance and report errors if any.
            let request = self
                .build()
                .map_err(|err| PubNubError::general_api_error(err.to_string(), None))?;

            let transport_request = request.transport_request();
            let client = request.pubnub_client.clone();
            let deserializer = request.deserializer;

            select! {
                _ = cancel_task.wait_for_cancel().fuse() => {
                    Err(PubNubError::EffectCanceled)
                },
                result = client
                    .transport
                    .send(transport_request)
                    .fuse() => {
                        result?
                            .body
                            .map(|bytes| deserializer.deserialize(&bytes))
                            .map_or(
                                Err(PubNubError::general_api_error(
                                    "No body in the response!",
                                    None,
                                )),
                                |response_body| {
                                    response_body.and_then::<SubscribeResult, _>(|body| body.try_into())
                                },
                            )
                    }
            }
        }

        // TODO: Handle decryption (when provider will be exposed).
    }
}

#[cfg(not(feature = "serde"))]
impl<T> SubscribeRequestWithDeserializerBuilder<T> {
    /// Add custom deserializer.
    ///
    /// Adds the deserializer to the [`SubscribeRequestBuilder`],
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub fn deserialize_with<D>(self, deserializer: D) -> SubscribeRequestBuilder<T, D>
    where
        T: Transport,
        D: for<'ds> Deserializer<'ds, SubscribeResponseBody> + Send,
    {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.pubnub_client),
            deserializer: Some(deserializer),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod should {
    use crate::{core::TransportResponse, PubNubClientBuilder};

    use super::*;

    #[tokio::test]
    async fn be_able_to_cancel_subscribe_call() {
        struct MockTransport;

        #[async_trait::async_trait]
        impl Transport for MockTransport {
            async fn send(&self, _req: TransportRequest) -> Result<TransportResponse, PubNubError> {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; // Simulate long request.

                Ok(TransportResponse::default())
            }
        }

        let (tx, rx) = async_channel::bounded(1);

        let cancel_task = CancellationTask::new(rx, "test".into());

        tx.send("test".into()).await.unwrap();

        let result = PubNubClientBuilder::with_transport(MockTransport)
            .with_keyset(crate::Keyset {
                subscribe_key: "test",
                publish_key: Some("test"),
                secret_key: None,
            })
            .with_user_id("test")
            .build()
            .unwrap()
            .subscribe_request()
            .channels(vec!["test".into()])
            .execute(cancel_task)
            .await;

        assert!(matches!(result, Err(PubNubError::EffectCanceled)));
    }
}
