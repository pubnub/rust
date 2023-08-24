//! # PubNub subscribe module.
//!
//! This module has all the builders for subscription to real-time updates from
//! a list of channels and channel groups.
//!
use derive_builder::Builder;
#[cfg(feature = "std")]
use futures::{
    future::BoxFuture,
    {select_biased, FutureExt},
};

use crate::{
    core::{
        blocking,
        utils::encoding::join_url_encoded,
        Deserializer, PubNubError, Transport, {TransportMethod, TransportRequest},
    },
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{builders, result::SubscribeResult, SubscribeCursor, SubscribeResponseBody},
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

#[cfg(feature = "std")]
use crate::{core::event_engine::cancel::CancellationTask, lib::alloc::sync::Arc};

/// The [`SubscribeRequestBuilder`] is used to build subscribe request which
/// will be used for real-time updates notification from the [`PubNub`] network.
///
/// This struct used by the [`subscribe`] method of the [`PubNubClient`].
/// The [`subscribe`] method is used to subscribe and receive real-time updates
/// from the [`PubNub`] network.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Debug, Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::subscribe)", validate = "Self::validate"),
    no_std
)]
pub(crate) struct SubscribeRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) pubnub_client: PubNubClientInstance<T, D>,

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
        setter(strip_option),
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

impl<T, D> SubscribeRequest<T, D> {
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

        query.insert("heartbeat".into(), self.heartbeat.to_string());

        TransportRequest {
            path: format!("/v2/subscribe/{sub_key}/{channels}/0"),
            query_parameters: query,
            method: TransportMethod::Get,
            ..Default::default()
        }
    }
}

impl<T, D> SubscribeRequestBuilder<T, D> {
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

#[cfg(feature = "std")]
impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    pub async fn execute_with_cancel_and_delay<F>(
        self,
        delay: Arc<F>,
        cancel_task: CancellationTask,
    ) -> Result<SubscribeResult, PubNubError>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        select_biased! {
            _ = cancel_task.wait_for_cancel().fuse() => {
                Err(PubNubError::EffectCanceled)
            },
            response = self.execute_with_delay(delay).fuse() => {
                response
            }
        }
    }

    pub async fn execute_with_delay<F>(self, delay: Arc<F>) -> Result<SubscribeResult, PubNubError>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        // Postpone request execution if required.
        delay().await;

        self.execute().await
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call request.
    pub async fn execute(self) -> Result<SubscribeResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send::<SubscribeResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call request.
    pub fn execute_blocking(self) -> Result<SubscribeResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request();
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<SubscribeResponseBody, _, _, _>(&client.transport, deserializer)
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod should {
    use super::*;
    use crate::{core::TransportResponse, PubNubClientBuilder};
    use futures::future::ready;

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
            .execute_with_cancel_and_delay(Arc::new(|| ready(()).boxed()), cancel_task)
            .await;

        assert!(matches!(result, Err(PubNubError::EffectCanceled)));
    }
}
