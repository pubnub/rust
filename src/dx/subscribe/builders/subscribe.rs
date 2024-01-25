//! # PubNub subscribe module.
//!
//! The [`SubscribeRequestBuilder`] lets you to make and execute request that
//! will receive real-time updates from a list of channels and channel groups.

use derive_builder::Builder;
#[cfg(feature = "std")]
use futures::{
    future::BoxFuture,
    {select_biased, FutureExt},
};

use crate::{
    core::{
        blocking,
        utils::encoding::{
            url_encode_extended, url_encoded_channel_groups, url_encoded_channels,
            UrlEncodeExtension,
        },
        Deserializer, PubNubError, Transport, {TransportMethod, TransportRequest},
    },
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{builders, result::SubscribeResult, SubscribeResponseBody, SubscriptionCursor},
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

#[cfg(all(feature = "presence", feature = "std"))]
use crate::lib::alloc::vec;
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
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::subscribe)", validate = "Self::validate"),
    no_std
)]
pub(crate) struct SubscribeRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
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
    pub(in crate::dx::subscribe) cursor: SubscriptionCursor,

    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// serialized `state` as values. State with heartbeat can be set **only**
    /// for channels.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # use pubnub::core::Serialize;
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let state = HashMap::<String, Vec<u8>>::from([(
    ///     "announce".to_string(),
    ///     HashMap::<String, bool>::from([
    ///         ("is_owner".to_string(), false),
    ///         ("is_admin".to_string(), true)
    ///     ]).serialize()?
    /// )]);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "presence")]
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom, strip_option),
        default = "None"
    )]
    pub(in crate::dx::subscribe) state: Option<Vec<u8>>,

    /// `user_id`presence timeout period.
    ///
    /// A heartbeat is a period of time during which `user_id` is visible
    /// `online`.
    /// If, within the heartbeat period, another heartbeat request or a
    /// subscribe (for an implicit heartbeat) request `timeout` will be
    /// announced for `user_id`.
    ///
    /// By default it is set to **300** seconds.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"))]
    pub(in crate::dx::subscribe) heartbeat: u64,

    /// Message filtering predicate.
    ///
    /// The [`PubNub`] network can filter out messages published with `meta`
    /// before they reach subscribers using these filtering expressions, which
    /// are based on the definition of the [`filter language`].
    ///
    /// [`PubNub`]:https://www.pubnub.com/
    /// [`filter language`]: https://www.pubnub.com/docs/general/messages/publish#filter-language-definition
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,
}

impl<T, D> SubscribeRequestBuilder<T, D> {
    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// nested `HashMap` with values. State with subscribe can be set **only**
    /// for channels.
    #[cfg(all(feature = "presence", feature = "std"))]
    pub(in crate::dx::subscribe) fn state(mut self, state: HashMap<String, Vec<u8>>) -> Self {
        let mut serialized_state = vec![b'{'];
        for (key, mut value) in state {
            serialized_state.append(&mut format!("\"{}\":", key).as_bytes().to_vec());
            serialized_state.append(&mut value);
            serialized_state.push(b',');
        }
        if serialized_state.last() == Some(&b',') {
            serialized_state.pop();
        }
        serialized_state.push(b'}');

        self.state = Some(Some(serialized_state));
        self
    }

    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// subscribe request instance.
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

    /// Build [`HeartbeatRequest`] from builder.
    fn request(self) -> Result<SubscribeRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> SubscribeRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::subscribe) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let config = &self.pubnub_client.config;
        let sub_key = &config.subscribe_key;
        let mut query: HashMap<String, String> = HashMap::new();
        query.extend::<HashMap<String, String>>(self.cursor.clone().into());

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|groups| query.insert("channel-group".into(), groups));

        #[cfg(feature = "presence")]
        if let Some(state) = self.state.as_ref() {
            let state_json =
                String::from_utf8(state.clone()).map_err(|err| PubNubError::Serialization {
                    details: err.to_string(),
                })?;
            query.insert("state".into(), state_json);
        }

        self.filter_expression
            .as_ref()
            .filter(|e| !e.is_empty())
            .and_then(|e| {
                query.insert(
                    "filter-expr".into(),
                    url_encode_extended(e.as_bytes(), UrlEncodeExtension::NonChannelPath),
                )
            });

        query.insert("heartbeat".into(), self.heartbeat.to_string());

        Ok(TransportRequest {
            path: format!(
                "/v2/subscribe/{sub_key}/{}/0",
                url_encoded_channels(&self.channels)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            #[cfg(feature = "std")]
            timeout: config.transport.subscribe_request_timeout,
            ..Default::default()
        })
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: Transport + 'static,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<SubscribeResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<SubscribeResponseBody, _, _, _>(
                &client.transport,
                deserializer,
                #[cfg(feature = "std")]
                &client.config.transport.retry_configuration,
                #[cfg(feature = "std")]
                &client.runtime,
            )
            .await
    }

    /// Build and call asynchronous request after delay.
    ///
    /// Perform delayed request call with ability to cancel it before call.
    #[cfg(feature = "std")]
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

    /// Build and call asynchronous request after configured delay.
    #[cfg(feature = "std")]
    async fn execute_with_delay<F>(self, delay: Arc<F>) -> Result<SubscribeResult, PubNubError>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        // Postpone request execution.
        delay().await;

        self.execute().await
    }
}

impl<T, D> SubscribeRequestBuilder<T, D>
where
    T: blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<SubscribeResult, PubNubError> {
        // Build request instance and report errors if any.
        let request = self
            .build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))?;

        let transport_request = request.transport_request()?;
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
