//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

#[cfg(feature = "std")]
use futures::{
    future::{ready, BoxFuture},
    FutureExt,
};
use spin::RwLock;

use crate::dx::{pubnub_client::PubNubClientInstance, subscribe::raw::RawSubscriptionBuilder};

#[doc(inline)]
pub use types::{
    File, MessageAction, Object, Presence, SubscribeCursor, SubscribeMessageType, SubscribeStatus,
    SubscribeStreamEvent,
};
pub mod types;

#[cfg(feature = "std")]
use crate::{
    core::{Deserializer, PubNubError, Transport},
    lib::alloc::{boxed::Box, string::String, sync::Arc, vec::Vec},
    subscribe::result::SubscribeResult,
};

#[cfg(feature = "std")]
use crate::core::{
    event_engine::{CancellationTask, EventEngine},
    runtime::Runtime,
};

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
pub use result::{SubscribeResponseBody, Update};
pub mod result;

#[cfg(feature = "std")]
pub(crate) use subscription_manager::SubscriptionManager;
#[cfg(feature = "std")]
pub(crate) mod subscription_manager;
use crate::subscribe::event_engine::SubscribeEventEngine;
#[cfg(feature = "std")]
#[doc(inline)]
use event_engine::{types::SubscriptionParams, SubscribeEffectHandler, SubscribeState};

#[cfg(feature = "std")]
pub(crate) mod event_engine;

#[cfg(feature = "std")]
impl<T, D> PubNubClientInstance<T, D>
where
    T: Transport + Send + 'static,
    D: Deserializer + 'static,
{
    /// Create subscription listener.
    ///
    /// Listeners configure [`PubNubClient`] to receive real-time updates for
    /// specified list of channels and groups.
    ///
    /// ```no_run // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// #   let client = PubNubClientBuilder::with_reqwest_transport()
    /// #      .with_keyset(Keyset {
    /// #          subscribe_key: "demo",
    /// #          publish_key: Some("demo"),
    /// #          secret_key: None,
    /// #      })
    /// #      .with_user_id("user_id")
    /// #      .build()?;
    /// client
    ///     .subscribe()
    ///     .channels(["hello".into(), "world".into()].to_vec())
    ///     .execute()?
    ///     .stream()
    ///     .for_each(|event| async move {
    ///         match event {
    ///             SubscribeStreamEvent::Update(update) => println!("update: {:?}", update),
    ///             SubscribeStreamEvent::Status(status) => println!("status: {:?}", status),
    ///         }
    ///     })
    ///     .await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(all(feature = "tokio", feature = "serde"))]
    pub fn subscribe(&self) -> SubscriptionBuilder {
        self.configure_subscribe();

        SubscriptionBuilder {
            subscription: Some(self.subscription.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn configure_subscribe(&self) -> Arc<RwLock<Option<SubscriptionManager>>> {
        {
            // Initialize subscription module when it will be first required.
            let mut slot = self.subscription.write();
            if slot.is_none() {
                *slot = Some(SubscriptionManager::new(self.subscribe_event_engine()));
                // *subscription_slot =
                // Some(self.clone().subscription_manager(runtime));
            }
        }

        self.subscription.clone()
    }

    pub(crate) fn subscribe_event_engine(&self) -> Arc<SubscribeEventEngine> {
        let channel_bound = 10; // TODO: Think about this value
        let emit_messages_client = self.clone();
        let emit_status_client = self.clone();
        let subscribe_client = self.clone();
        let request_retry_delay_policy = self.config.retry_policy.clone();
        let request_retry_policy = self.config.retry_policy.clone();
        let runtime = self.runtime.clone();
        let runtime_sleep = runtime.clone();

        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);

        EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |params| {
                    let delay_in_secs = request_retry_delay_policy
                        .retry_delay(&params.attempt, params.reason.as_ref());
                    let inner_runtime_sleep = runtime_sleep.clone();

                    Self::subscribe_call(
                        subscribe_client.clone(),
                        params.clone(),
                        Arc::new(move || {
                            if let Some(delay) = delay_in_secs {
                                inner_runtime_sleep.clone().sleep(delay).boxed()
                            } else {
                                ready(()).boxed()
                            }
                        }),
                        cancel_rx.clone(),
                    )
                }),
                Arc::new(move |status| Self::emit_status(emit_status_client.clone(), &status)),
                Arc::new(Box::new(move |updates| {
                    Self::emit_messages(emit_messages_client.clone(), updates)
                })),
                request_retry_policy,
                cancel_tx,
            ),
            SubscribeState::Unsubscribed,
            runtime,
        )
    }

    pub(crate) fn subscribe_call<F>(
        client: Self,
        params: SubscriptionParams,
        delay: Arc<F>,
        cancel_rx: async_channel::Receiver<String>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let mut request = client
            .subscribe_request()
            .cursor(params.cursor.cloned().unwrap_or_default()); // TODO: is this clone required?

        if let Some(channels) = params.channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = params.channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }
        let cancel_task = CancellationTask::new(cancel_rx, params.effect_id.to_owned()); // TODO: needs to be owned?

        request
            .execute_with_cancel_and_delay(delay, cancel_task)
            .boxed()
    }

    fn emit_status(client: Self, status: &SubscribeStatus) {
        if let Some(manager) = client.subscription.read().as_ref() {
            manager.notify_new_status(status)
        }
    }

    fn emit_messages(client: Self, messages: Vec<Update>) {
        let messages = if let Some(cryptor) = &client.cryptor {
            messages
                .into_iter()
                .map(|update| update.decrypt(cryptor))
                .collect()
        } else {
            messages
        };

        if let Some(manager) = client.subscription.read().as_ref() {
            manager.notify_new_messages(messages)
        }
    }
}

impl<T, D> PubNubClientInstance<T, D> {
    /// Create subscription listener.
    ///
    /// Listeners configure [`PubNubClient`] to receive real-time updates for
    /// specified list of channels and groups.
    ///
    /// ```no_run // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use pubnub::{Keyset, PubNubClientBuilder};
    /// #
    /// #   let client = PubNubClientBuilder::with_reqwest_transport()
    /// #      .with_keyset(Keyset {
    /// #          subscribe_key: "demo",
    /// #          publish_key: Some("demo"),
    /// #          secret_key: None,
    /// #      })
    /// #      .with_user_id("user_id")
    /// #      .build()?;
    /// client
    ///     .subscribe_raw()
    ///     .channels(["hello".into(), "world".into()].to_vec())
    ///     .execute()?
    ///     .stream()
    ///     .for_each(|update| async move {
    ///           println!("Received update: {:?}", update);
    ///     })
    ///     .await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    pub fn subscribe_raw(&self) -> RawSubscriptionBuilder<T, D> {
        RawSubscriptionBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub(crate) fn subscribe_request(&self) -> SubscribeRequestBuilder<T, D> {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::{blocking, PubNubError, TransportRequest, TransportResponse},
        providers::deserialization_serde::DeserializerSerde,
        Keyset, PubNubClientBuilder, PubNubGenericClient,
    };

    struct MockTransport;

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        async fn send(&self, _request: TransportRequest) -> Result<TransportResponse, PubNubError> {
            Ok(TransportResponse {
                status: 200,
                headers: [].into(),
                body: generate_body(),
            })
        }
    }

    impl blocking::Transport for MockTransport {
        fn send(&self, _req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            Ok(TransportResponse {
                status: 200,
                headers: [].into(),
                body: generate_body(),
            })
        }
    }

    fn generate_body() -> Option<Vec<u8>> {
        Some(
            r#"{
                        "t": {
                            "t": "15628652479932717",
                            "r": 4
                        },
                        "m": [
                            { "a": "1",
                            "f": 514,
                            "i": "pn-0ca50551-4bc8-446e-8829-c70b704545fd",
                            "s": 1,
                            "p": {
                            "t": "15628652479933927",
                            "r": 4
                            },
                            "k": "demo",
                            "c": "my-channel",
                            "d": "my message",
                            "b": "my-channel"
                            }
                        ]
                    }"#
            .into(),
        )
    }

    fn client() -> PubNubGenericClient<MockTransport, DeserializerSerde> {
        PubNubClientBuilder::with_transport(MockTransport)
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: Some("demo"),
                secret_key: None,
            })
            .with_user_id("user")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn create_builder() {
        let _ = client().subscribe();
    }

    #[tokio::test]
    async fn subscribe() {
        let subscription = client()
            .subscribe()
            .channels(["world".into()].to_vec())
            .execute()
            .unwrap();

        use futures::StreamExt;
        let status = subscription.stream().next().await.unwrap();

        assert!(matches!(
            status,
            SubscribeStreamEvent::Status(SubscribeStatus::Connected)
        ));
    }

    #[tokio::test]
    async fn subscribe_raw() {
        let subscription = client()
            .subscribe_raw()
            .channels(["world".into()].to_vec())
            .execute()
            .unwrap();

        use futures::StreamExt;
        let message = subscription.stream().boxed().next().await;

        assert!(message.is_some());
    }

    #[test]
    fn subscribe_raw_blocking() {
        let subscription = client()
            .subscribe_raw()
            .channels(["world".into()].to_vec())
            .execute_blocking()
            .unwrap();

        let message = subscription.iter().next();

        assert!(message.is_some());
    }
}
