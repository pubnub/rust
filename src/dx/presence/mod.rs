//! # Presence module.
//!
//! The presence module allows retrieving presence information and managing the
//! state in specific channels associated with specific `uuid`.
//! The presence module contains [`SetStateRequestBuilder`] type.

#[cfg(feature = "std")]
use futures::{
    future::{ready, BoxFuture},
    {select_biased, FutureExt},
};
use spin::RwLock;

use crate::{
    core::{Deserializer, PubNubError, Serialize, Transport},
    dx::pubnub_client::PubNubClientInstance,
};

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
pub use result::{HeartbeatResponseBody, HeartbeatResult, LeaveResponseBody, LeaveResult};
pub mod result;

#[cfg(feature = "std")]
#[doc(inline)]
pub(crate) use presence_manager::PresenceManager;
#[cfg(feature = "std")]
pub(crate) mod presence_manager;
#[cfg(feature = "std")]
#[doc(inline)]
pub(crate) use event_engine::{
    types::PresenceParameters, PresenceEffectHandler, PresenceEventEngine, PresenceState,
};
#[cfg(feature = "std")]
pub(crate) mod event_engine;
#[cfg(feature = "std")]
use crate::{
    core::{
        event_engine::{cancel::CancellationTask, EventEngine},
        Runtime,
    },
    lib::alloc::sync::Arc,
};

impl<T, D> PubNubClientInstance<T, D> {
    /// Create a heartbeat request builder.
    ///
    /// This method is used to announce the presence of `user_id` on the
    /// provided list of channels and/or groups.
    ///
    /// Instance of [`HeartbeatRequestsBuilder`] returned.
    ///
    /// # Example
    /// ```rust
    /// use pubnub::presence::*;
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut pubnub = // PubNubClient
    /// #         PubNubClientBuilder::with_reqwest_transport()
    /// #             .with_keyset(Keyset {
    /// #                 subscribe_key: "demo",
    /// #                 publish_key: None,
    /// #                 secret_key: None
    /// #             })
    /// #             .with_user_id("uuid")
    /// #             .build()?;
    /// pubnub
    ///     .heartbeat()
    ///     .channels(["lobby".into(), "announce".into()])
    ///     .channel_groups(["area-51".into()])
    ///     .state(HashMap::<String, HashMap<String, bool>>::from(
    ///         [(
    ///             "lobby".into(),
    ///             HashMap::from([("is_admin".into(), false)])
    ///         )]
    ///     ))
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn heartbeat(&self) -> HeartbeatRequestBuilder<T, D> {
        HeartbeatRequestBuilder {
            pubnub_client: Some(self.clone()),
            heartbeat: Some(self.config.heartbeat_value),
            user_id: Some(self.config.user_id.clone().to_string()),
            ..Default::default()
        }
    }

    /// Create a leave request builder.
    ///
    /// This method is used to announce `leave` of `user_id` on the provided
    /// list of channels and/or groups and update state associated with
    /// `user_id` on channels.
    ///
    /// Instance of [`LeaveRequestBuilder`] returned.
    pub(in crate::dx::presence) fn leave(&self) -> LeaveRequestBuilder<T, D> {
        LeaveRequestBuilder {
            pubnub_client: Some(self.clone()),
            user_id: Some(self.config.user_id.clone().to_string()),
            ..Default::default()
        }
    }

    /// Create a set state request builder.
    ///
    /// This method is used to update state associated with `user_id` on
    /// channels and and channels registered with channel groups.
    ///
    /// Instance of [`SetStateRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust
    /// use pubnub::presence::*;
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// let mut pubnub = // PubNubClient
    /// #         PubNubClientBuilder::with_reqwest_transport()
    /// #             .with_keyset(Keyset {
    /// #                 subscribe_key: "demo",
    /// #                 publish_key: None,
    /// #                 secret_key: None
    /// #             })
    /// #             .with_user_id("uuid")
    /// #             .build()?;
    /// pubnub
    ///     .set_presence_state(HashMap::<String, bool>::from(
    ///          [("is_admin".into(), false)]
    ///      ))
    ///     .channels(["lobby".into(), "announce".into()])
    ///     .channel_groups(["area-51".into()])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_presence_state<S>(&self, state: S) -> SetStateRequestBuilder<T, D>
    where
        S: Serialize + Send + Sync + 'static,
    {
        SetStateRequestBuilder {
            pubnub_client: Some(self.clone()),
            state: Some(state.serialize().ok()),
            user_id: Some(self.config.user_id.clone().to_string()),
            ..Default::default()
        }
    }

    /// Create a heartbeat request builder.
    ///
    /// This method is used to update state associated with `user_id` on
    /// channels using `heartbeat` operation endpoint.
    ///
    /// Instance of [`HeartbeatRequestsBuilder`] returned.
    pub fn set_state_with_heartbeat<U>(&self, state: U) -> HeartbeatRequestBuilder<T, D>
    where
        U: Serialize + Send + Sync + 'static,
    {
        self.heartbeat().state(state)
    }

    /// Create a get state request builder.
    ///
    /// This method is used to get state associated with `user_id` on
    /// channels and and channels registered with channel groups.
    ///
    /// Instance of [`SetStateRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust
    /// use pubnub::presence::*;
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// let mut pubnub = // PubNubClient
    /// #         PubNubClientBuilder::with_reqwest_transport()
    /// #             .with_keyset(Keyset {
    /// #                 subscribe_key: "demo",
    /// #                 publish_key: None,
    /// #                 secret_key: None
    /// #             })
    /// #             .with_user_id("uuid")
    /// #             .build()?;
    /// pubnub
    ///     .get_presence_state()
    ///     .channels(["lobby".into(), "announce".into()])
    ///     .channel_groups(["area-51".into()])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_presence_state(&self) -> GetStateRequestBuilder<T, D> {
        GetStateRequestBuilder {
            pubnub_client: Some(self.clone()),
            user_id: Some(self.config.user_id.clone().to_string()),
            ..Default::default()
        }
    }

    /// Create a here now request builder.
    ///
    /// This method is used to get information about current occupancy of
    /// channels and channel groups.
    ///
    /// Instance of [`HereNowRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust
    /// use pubnub::presence::*;
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// let mut pubnub = // PubNubClient
    /// #         PubNubClientBuilder::with_reqwest_transport()
    /// #             .with_keyset(Keyset {
    /// #                 subscribe_key: "demo",
    /// #                 publish_key: None,
    /// #                 secret_key: None,
    /// #             })
    /// #             .with_user_id("uuid")
    /// #             .build()?;
    /// let response = pubnub.here_now()
    ///         .channels(["lobby".into()])
    ///         .include_state(true)
    ///         .include_user_id(true)
    ///         .execute()
    ///         .await?;
    ///
    /// println!("All channels data: {:?}", response);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn here_now(&self) -> HereNowRequestBuilder<T, D> {
        HereNowRequestBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }

    /// Create a where now request builder.
    ///
    /// This method is used to get information about channels where `user_id`
    /// is currently present.
    ///
    /// Instance of [`WhereNowRequestBuilder`] returned.
    ///
    /// # Example
    /// ```rust
    /// use pubnub::presence::*;
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// # use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// let mut pubnub = // PubNubClient
    /// #         PubNubClientBuilder::with_reqwest_transport()
    /// #             .with_keyset(Keyset {
    /// #                 subscribe_key: "demo",
    /// #                 publish_key: None,
    /// #                 secret_key: None,
    /// #             })
    /// #             .with_user_id("uuid")
    /// #             .build()?;
    /// let response = pubnub.where_now().user_id("user_id").execute().await?;

    ///
    /// println!("User channels: {:?}", response);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn where_now(&self) -> WhereNowRequestBuilder<T, D> {
        WhereNowRequestBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }
}

impl<T, D> PubNubClientInstance<T, D>
where
    T: Transport + Send + 'static,
    D: Deserializer + 'static,
{
    /// Announce `join` for `user_id` on provided channels and groups.
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    pub(crate) fn announce_join(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.configure_presence();

        {
            let slot = self.presence.read();
            if let Some(presence) = slot.as_ref() {
                presence.announce_join(channels, channel_groups);
            }
        };
    }

    /// Announce `leave` for `user_id` on provided channels and groups.
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    pub(crate) fn announce_left(
        &self,
        channels: Option<Vec<String>>,
        channel_groups: Option<Vec<String>>,
    ) {
        self.configure_presence();

        {
            let slot = self.presence.read();
            if let Some(presence) = slot.as_ref() {
                presence.announce_left(channels, channel_groups);
            }
        };
    }

    /// Complete presence configuration.
    ///
    /// Presence configuration used only with presence event engine.
    #[cfg(feature = "std")]
    pub(crate) fn configure_presence(&self) -> Arc<RwLock<Option<PresenceManager>>> {
        {
            let mut slot = self.presence.write();
            if slot.is_none() {
                *slot = Some(PresenceManager::new(self.presence_event_engine(), None));
            }
        }

        self.presence.clone()
    }

    /// Presence event engine.
    ///
    /// Prepare presence event engine instance which will be used for `user_id`
    /// presence announcement and management.
    #[cfg(feature = "std")]
    pub(crate) fn presence_event_engine(&self) -> Arc<PresenceEventEngine> {
        let channel_bound = 3;
        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);
        let delayed_heartbeat_cancel_rx = cancel_rx.clone();
        let wait_cancel_rx = cancel_rx.clone();
        let runtime = self.runtime.clone();
        let delayed_heartbeat_call_client = self.clone();
        let heartbeat_call_client = self.clone();
        let leave_call_client = self.clone();
        let wait_call_client = self.clone();
        let request_retry_delay_policy = self.config.retry_policy.clone();
        let request_retry_policy = self.config.retry_policy.clone();
        let delayed_heartbeat_runtime_sleep = runtime.clone();
        let wait_runtime_sleep = runtime.clone();

        EventEngine::new(
            PresenceEffectHandler::new(
                Arc::new(move |parameters| {
                    Self::heartbeat_call(heartbeat_call_client.clone(), parameters.clone())
                }),
                Arc::new(move |parameters| {
                    let delay_in_secs = request_retry_delay_policy
                        .retry_delay(&parameters.attempt, parameters.reason.as_ref());
                    let inner_runtime_sleep = delayed_heartbeat_runtime_sleep.clone();

                    Self::delayed_heartbeat_call(
                        delayed_heartbeat_call_client.clone(),
                        parameters.clone(),
                        Arc::new(move || {
                            if let Some(delay) = delay_in_secs {
                                inner_runtime_sleep.clone().sleep(delay).boxed()
                            } else {
                                ready(()).boxed()
                            }
                        }),
                        delayed_heartbeat_cancel_rx.clone(),
                    )
                }),
                Arc::new(move |parameters| {
                    Self::leave_call(leave_call_client.clone(), parameters.clone())
                }),
                Arc::new(move |effect_id| {
                    let delay_in_secs = wait_call_client.config.heartbeat_interval;
                    let inner_runtime_sleep = wait_runtime_sleep.clone();

                    Self::wait_call(
                        effect_id,
                        Arc::new(move || {
                            if let Some(delay) = delay_in_secs {
                                inner_runtime_sleep.clone().sleep(delay).boxed()
                            } else {
                                ready(()).boxed()
                            }
                        }),
                        wait_cancel_rx.clone(),
                    )
                }),
                request_retry_policy,
                cancel_tx,
            ),
            PresenceState::Inactive,
            runtime,
        )
    }

    /// Call to announce `user_id` presence.
    #[cfg(feature = "std")]
    pub(crate) fn heartbeat_call(
        client: Self,
        params: PresenceParameters,
    ) -> BoxFuture<'static, Result<HeartbeatResult, PubNubError>> {
        client.heartbeat_request(params).execute().boxed()
    }

    /// Call delayed announce of `user_id` presence.
    #[cfg(feature = "std")]
    pub(crate) fn delayed_heartbeat_call<F>(
        client: Self,
        params: PresenceParameters,
        delay: Arc<F>,
        cancel_rx: async_channel::Receiver<String>,
    ) -> BoxFuture<'static, Result<HeartbeatResult, PubNubError>>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let effect_id = params.effect_id.to_owned();
        let cancel_task = CancellationTask::new(cancel_rx, effect_id);

        client
            .heartbeat_request(params)
            .execute_with_cancel_and_delay(delay, cancel_task)
            .boxed()
    }

    /// Call announce `leave` for `user_id`.
    #[cfg(feature = "std")]
    pub(crate) fn leave_call(
        client: Self,
        params: PresenceParameters,
    ) -> BoxFuture<'static, Result<LeaveResult, PubNubError>> {
        let mut request = client.leave();

        if let Some(channels) = params.channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = params.channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }

        request.execute().boxed()
    }

    /// Heartbeat idle.
    #[cfg(feature = "std")]
    pub(crate) fn wait_call<F>(
        effect_id: &str,
        delay: Arc<F>,
        cancel_rx: async_channel::Receiver<String>,
    ) -> BoxFuture<'static, Result<(), PubNubError>>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let cancel_task = CancellationTask::new(cancel_rx, effect_id.to_owned());

        async move {
            select_biased! {
                _ = cancel_task.wait_for_cancel().fuse() => {
                    Err(PubNubError::EffectCanceled)
                },
                _ = delay().fuse() => Ok(())
            }
        }
        .boxed()
    }

    /// Call to update `state` associated with `user_id`.
    #[allow(dead_code)]
    pub(crate) fn set_heartbeat_call<U>(client: Self, _params: PresenceParameters, state: U)
    where
        U: Serialize + Send + Sync + 'static,
    {
        // TODO: This is still under development and will be part of EE.
        #[cfg(feature = "std")]
        {
            client.configure_presence();

            let state = state.serialize().ok();
            if let Some(presence) = client.presence.clone().write().as_mut() {
                presence.state = state;
            }
        }
    }

    pub(crate) fn heartbeat_request(
        &self,
        params: PresenceParameters,
    ) -> HeartbeatRequestBuilder<T, D> {
        let mut request = self.heartbeat();

        if let Some(channels) = params.channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = params.channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }

        if let Some(presence) = self.presence.clone().read().as_ref() {
            request = request.state_serialized(presence.state.clone())
        }

        request
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::core::{PubNubError, Transport, TransportRequest, TransportResponse};
    use crate::providers::deserialization_serde::DeserializerSerde;
    use crate::transport::middleware::PubNubMiddleware;
    use crate::{lib::collections::HashMap, Keyset, PubNubClientBuilder};

    /// Requests handler function type.
    type RequestHandler = Box<dyn Fn(&TransportRequest) + Send + Sync>;

    #[derive(Default)]
    struct MockTransport {
        ///  Response which mocked transport should return.
        response: Option<TransportResponse>,

        /// Request handler function which will be called before returning
        /// response.
        ///
        /// Use function to verify request parameters.
        request_handler: Option<RequestHandler>,
    }

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        async fn send(&self, req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            // Calling request handler (if provided).
            if let Some(handler) = &self.request_handler {
                handler(&req);
            }

            Ok(self.response.clone().unwrap_or(transport_response(200)))
        }
    }

    /// Service response payload.
    fn transport_response(status: u16) -> TransportResponse {
        TransportResponse {
            status,
            body: Some(Vec::from(if status < 400 {
                "{\"status\": 200, \"message\": \"OK\", \"service\": \"Presence\"}"
            } else {
                "\"error\":{{\"message\":\"Overall error\",\"source\":\"test\",\"details\":[{{\"message\":\"Error\",\"location\":\"signature\",\"locationType\":\"query\"}}]}}"
            })),
            ..Default::default()
        }
    }

    /// Construct test client with mocked transport.
    fn client(
        with_subscribe_key: bool,
        transport: Option<MockTransport>,
    ) -> PubNubClientInstance<PubNubMiddleware<MockTransport>, DeserializerSerde> {
        PubNubClientBuilder::with_transport(transport.unwrap_or(MockTransport {
            response: None,
            request_handler: None,
        }))
        .with_keyset(Keyset {
            subscribe_key: if with_subscribe_key { "demo" } else { "" },
            publish_key: None,
            secret_key: None,
        })
        .with_user_id("user")
        .build()
        .unwrap()
    }

    #[test]
    fn not_heartbeat_when_subscribe_key_missing() {
        let client = client(false, None);
        let request = client.heartbeat().channels(["test".into()]).build();

        assert!(&client.config.subscribe_key.is_empty());
        assert!(request.is_err())
    }

    #[tokio::test]
    async fn send_heartbeat() {
        let client = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: None,
            })
            .with_user_id("user_id")
            .build()
            .unwrap();

        let result = client
            .heartbeat()
            .state(HashMap::<String, HashMap<String, bool>>::from([(
                "hello".into(),
                HashMap::from([("is_admin".into(), false)]),
            )]))
            .channels(["hello".into()])
            .user_id("my_user")
            .execute()
            .await;

        match result {
            Ok(_) => {}
            Err(err) => panic!("Request should not fail: {err}"),
        }
    }

    #[tokio::test]
    async fn include_state_in_query() {
        let transport = MockTransport {
            response: None,
            request_handler: Some(Box::new(|req| {
                assert!(req.query_parameters.contains_key("state"));
                assert!(req.query_parameters.get("state").is_some());

                let state = req.query_parameters.get("state").unwrap();
                assert!(state.contains("channel_a"));
                assert!(state.contains("channel_c"));
            })),
        };

        let _ = client(true, Some(transport))
            .heartbeat()
            .state(HashMap::<String, HashMap<String, String>>::from([
                (
                    "channel_a".into(),
                    HashMap::<String, String>::from([("value_a".into(), "secret_a".into())]),
                ),
                (
                    "channel_c".into(),
                    HashMap::<String, String>::from([("value_c".into(), "secret_c".into())]),
                ),
            ]))
            .channels(["channel_a".into(), "channel_b".into(), "channel_c".into()])
            .execute()
            .await;
    }
}
