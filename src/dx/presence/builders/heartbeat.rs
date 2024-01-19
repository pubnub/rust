//! # PubNub heartbeat module.
//!
//! The [`HeartbeatRequestBuilder`] lets you make and execute requests that will
//! announce specified `user_id` presence in the provided channels and groups.

use derive_builder::Builder;
#[cfg(feature = "std")]
use futures::{
    future::BoxFuture,
    {select_biased, FutureExt},
};

use crate::{
    core::{
        blocking,
        utils::{
            encoding::{url_encoded_channel_groups, url_encoded_channels},
            headers::{APPLICATION_JSON, CONTENT_TYPE},
        },
        Deserializer, PubNubError, Transport, TransportMethod, TransportRequest,
    },
    dx::{
        presence::{builders, HeartbeatResponseBody, HeartbeatResult},
        pubnub_client::PubNubClientInstance,
    },
    lib::{
        alloc::{
            format,
            string::{String, ToString},
            vec,
            vec::Vec,
        },
        collections::HashMap,
    },
};

#[cfg(feature = "std")]
use crate::{core::event_engine::cancel::CancellationTask, lib::alloc::sync::Arc};

/// The [`HeartbeatRequestsBuilder`] is used to build a `user_id` presence
/// announcement request that is sent to the [`PubNub`] network.
///
/// This struct is used by the [`heartbeat`] and [`set_state_with_heartbeat`]
/// methods of the [`PubNubClient`].
/// The [`heartbeat`] method is used to announce specified `user_id` presence in
/// the provided channels and groups.
/// The [`set_state_with_heartbeat`] is used to update the state associated with
/// `user_id` and announce its presence in the provided channels and groups.
///
/// [`PubNub`]:https://www.pubnub.com/
#[derive(Builder)]
#[builder(
    pattern = "owned",
    build_fn(vis = "pub(in crate::dx::presence)", validate = "Self::validate"),
    no_std
)]
pub struct HeartbeatRequest<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(custom))]
    pub(in crate::dx::presence) pubnub_client: PubNubClientInstance<T, D>,

    /// Channel(s) for announcement.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option, into),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channels: Vec<String>,

    /// Channel group(s) for announcement.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(into, strip_option),
        default = "vec![]"
    )]
    pub(in crate::dx::presence) channel_groups: Vec<String>,

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
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(custom, strip_option),
        default = "None"
    )]
    pub(in crate::dx::presence) state: Option<Vec<u8>>,

    /// `user_id`presence timeout period.
    ///
    /// A heartbeat is a period of time during which `user_id` is visible
    /// `online`.
    /// If, within the heartbeat period, another heartbeat request or subscribe
    /// (for an implicit heartbeat) request `timeout` will be announced for
    /// `user_id`.
    ///
    /// By default, it is set to **300** seconds.
    #[builder(
        field(vis = "pub(in crate::dx::presence)"),
        setter(strip_option),
        default = "300"
    )]
    pub(in crate::dx::presence) heartbeat: u64,

    /// Identifier for which presence in channels and/or channel groups will be
    /// announced.
    #[builder(field(vis = "pub(in crate::dx::presence)"), setter(strip_option, into))]
    pub(in crate::dx::presence) user_id: String,
}

impl<T, D> HeartbeatRequestBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that provided information is enough to build valid
    /// heartbeat request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        builders::validate_configuration(&self.pubnub_client).and_then(|_| {
            if channels_len == groups_len && channels_len == 0 {
                Err("Either channels or channel groups should be provided".into())
            } else if self.user_id.is_none() {
                Err("User id is missing".into())
            } else {
                Ok(())
            }
        })
    }

    /// Build [`HeartbeatRequest`] from builder.
    fn request(self) -> Result<HeartbeatRequest<T, D>, PubNubError> {
        self.build()
            .map_err(|err| PubNubError::general_api_error(err.to_string(), None, None))
    }
}

impl<T, D> HeartbeatRequest<T, D> {
    /// Create transport request from the request builder.
    pub(in crate::dx::presence) fn transport_request(
        &self,
    ) -> Result<TransportRequest, PubNubError> {
        let config = &self.pubnub_client.config;
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("heartbeat".into(), self.heartbeat.to_string());
        query.insert("uuid".into(), self.user_id.to_string());

        // Serialize list of channel groups and add into query parameters list.
        url_encoded_channel_groups(&self.channel_groups)
            .and_then(|groups| query.insert("channel-group".into(), groups));

        if let Some(state) = self.state.as_ref() {
            let state_json =
                String::from_utf8(state.clone()).map_err(|err| PubNubError::Serialization {
                    details: err.to_string(),
                })?;
            query.insert("state".into(), state_json);
        }

        Ok(TransportRequest {
            path: format!(
                "/v2/presence/sub_key/{}/channel/{}/heartbeat",
                &config.subscribe_key,
                url_encoded_channels(&self.channels)
            ),
            query_parameters: query,
            method: TransportMethod::Get,
            headers: [(CONTENT_TYPE.to_string(), APPLICATION_JSON.to_string())].into(),
            body: None,
            #[cfg(feature = "std")]
            timeout: config.transport.request_timeout,
        })
    }
}

impl<T, D> HeartbeatRequestBuilder<T, D> {
    /// A state that should be associated with the `user_id`.
    ///
    /// `state` object should be a `HashMap` with channel names as keys and
    /// nested `HashMap` with values. State with heartbeat can be set **only**
    /// for channels.
    ///
    /// # Example:
    /// ```rust,no_run
    /// # use std::collections::HashMap;
    /// # use pubnub::core::Serialize;
    /// # fn main() -> Result<(), pubnub::core::PubNubError> {
    /// let state: HashMap<String, HashMap<String, bool>> = HashMap::from([(
    ///     "announce".to_string(),
    ///     HashMap::<String, bool>::from([
    ///         ("is_owner".to_string(), false),
    ///         ("is_admin".to_string(), true)
    ///     ])
    /// )]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn state(mut self, state: HashMap<String, Vec<u8>>) -> Self {
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
}

#[allow(dead_code)]
impl<T, D> HeartbeatRequestBuilder<T, D>
where
    T: Transport,
    D: Deserializer + 'static,
{
    /// Build and call asynchronous request.
    pub async fn execute(self) -> Result<HeartbeatResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();

        transport_request
            .send::<HeartbeatResponseBody, _, _, _>(&client.transport, deserializer)
            .await
    }

    /// Build and call asynchronous request after delay.
    ///
    /// Perform delayed request call with ability to cancel it before call.
    #[cfg(feature = "std")]
    pub(in crate::dx::presence) async fn execute_with_cancel_and_delay<F>(
        self,
        delay: Arc<F>,
        cancel_task: CancellationTask,
    ) -> Result<HeartbeatResult, PubNubError>
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
    async fn execute_with_delay<F>(self, delay: Arc<F>) -> Result<HeartbeatResult, PubNubError>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        // Postpone request execution.
        delay().await;

        self.execute().await
    }
}

#[allow(dead_code)]
#[cfg(feature = "blocking")]
impl<T, D> HeartbeatRequestBuilder<T, D>
where
    T: blocking::Transport,
    D: Deserializer + 'static,
{
    /// Build and call synchronous request.
    pub fn execute_blocking(self) -> Result<HeartbeatResult, PubNubError> {
        let request = self.request()?;
        let transport_request = request.transport_request()?;
        let client = request.pubnub_client.clone();
        let deserializer = client.deserializer.clone();
        transport_request
            .send_blocking::<HeartbeatResponseBody, _, _, _>(&client.transport, deserializer)
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{core::TransportResponse, PubNubClientBuilder};
    use futures::future::ready;

    #[tokio::test]
    async fn be_able_to_cancel_delayed_heartbeat_call() {
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
            .heartbeat()
            .channels(vec!["test".into()])
            .execute_with_cancel_and_delay(Arc::new(|| ready(()).boxed()), cancel_task)
            .await;

        assert!(matches!(result, Err(PubNubError::EffectCanceled)));
    }

    // TODO: Make request cancelable
    // #[cfg(feature = "std")]
    // #[tokio::test]
    // async fn be_able_to_cancel_request() {
    //     struct MockTransport;
    //
    //     #[async_trait::async_trait]
    //     impl Transport for MockTransport {
    //         async fn send(&self, _req: TransportRequest) ->
    // Result<TransportResponse, PubNubError> {
    // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; //
    // Simulate long request.
    //
    //             Ok(TransportResponse::default())
    //         }
    //     }
    //
    //     let client = PubNubClientBuilder::with_transport(MockTransport)
    //         .with_keyset(crate::Keyset {
    //             subscribe_key: "test",
    //             publish_key: Some("test"),
    //             secret_key: None,
    //         })
    //         .with_user_id("test")
    //         .build()
    //         .unwrap();
    //     let _ = &client
    //         .detached_guard
    //         .notify_channel_tx
    //         .send_blocking(1)
    //         .unwrap();
    //
    //     let result = client
    //         .heartbeat()
    //         .channels(vec!["test".into()])
    //         .execute()
    //         .await;
    //
    //     assert!(matches!(result, Err(PubNubError::RequestCancel { .. })));
    // }
}
