//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

#[cfg(feature = "std")]
pub(crate) mod event_engine;
#[cfg(feature = "std")]
use event_engine::{SubscribeEffectHandler, SubscribeState};

#[cfg(feature = "std")]
use futures::{
    future::{ready, BoxFuture},
    FutureExt,
};

#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;

pub use result::{SubscribeResponseBody, Update};
pub mod result;

#[doc(inline)]
pub use types::{
    File, MessageAction, Object, Presence, SubscribeCursor, SubscribeMessageType, SubscribeStatus,
    SubscribeStreamEvent,
};
pub mod types;

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::sync::Arc};

#[cfg(feature = "std")]
use crate::{
    core::{PubNubError, Transport},
    lib::alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec},
    subscribe::result::SubscribeResult,
};

#[cfg(feature = "std")]
use crate::core::{event_engine::EventEngine, runtime::Runtime};

#[cfg(feature = "std")]
pub(crate) use subscription_manager::SubscriptionManager;
#[cfg(feature = "std")]
pub(crate) mod subscription_manager;

#[cfg(feature = "std")]
pub(crate) use subscription_configuration::{
    SubscriptionConfiguration, SubscriptionConfigurationRef,
};
#[cfg(feature = "std")]
pub(crate) mod subscription_configuration;

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[cfg(feature = "std")]
#[doc(inline)]
use cancel::CancellationTask;
#[cfg(feature = "std")]
mod cancel;

#[cfg(feature = "serde")]
use self::raw::RawSubscriptionBuilder;
#[cfg(not(feature = "serde"))]
use self::raw::RawSubscriptionWithDeserializerBuilder;

#[cfg(feature = "std")]
#[derive(Clone)]
pub(crate) struct SubscriptionParams<'execution> {
    channels: &'execution Option<Vec<String>>,
    channel_groups: &'execution Option<Vec<String>>,
    cursor: Option<&'execution SubscribeCursor>,
    attempt: u8,
    reason: Option<PubNubError>,
    effect_id: &'execution str,
}

#[cfg(feature = "std")]
impl<T> PubNubClientInstance<T>
where
    T: Transport + Send + 'static,
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
    ///
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(all(feature = "tokio", feature = "serde"))]
    pub fn subscribe(&self) -> SubscriptionBuilder {
        use crate::providers::futures_tokio::TokioRuntime;

        self.subscribe_with_runtime(TokioRuntime)
    }

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
    ///    .await;
    /// # Ok(())
    /// # }
    ///
    /// ```
    ///
    /// Instance of [`SubscriptionWithDeserializerBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(all(feature = "tokio", not(feature = "serde")))]
    pub fn subscribe(&self) -> SubscriptionWithDeserializerBuilder {
        use crate::providers::futures_tokio::TokioRuntime;

        self.subscribe_with_runtime(TokioRuntime)
    }

    /// Create subscription listener.
    ///
    /// Listeners configure [`PubNubClient`] to receive real-time updates for
    /// specified list of channels and groups.
    ///
    /// It takes custom runtime which will be used for detached tasks spawning
    /// and delayed task execution.
    ///
    /// ```no_run // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
    /// use pubnub::core::runtime::Runtime;
    /// use std::future::Future;
    ///
    /// #[derive(Clone)]
    /// struct MyRuntime;
    ///
    /// #[async_trait::async_trait]
    /// impl Runtime for MyRuntime {
    ///    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
    ///       // spawn the Future
    ///       // e.g. tokio::spawn(future);
    ///    }
    ///    
    ///    async fn sleep(self, _delay: u64) {
    ///       // e.g. tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await
    ///    }
    /// }
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
    ///
    /// client
    ///     .subscribe_with_runtime(MyRuntime)
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
    ///
    /// ```
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(feature = "serde")]
    pub fn subscribe_with_runtime<R>(&self, runtime: R) -> SubscriptionBuilder
    where
        R: Runtime + Send + Sync + 'static,
    {
        {
            // Initialize subscription module when it will be first required.
            let mut subscription_slot = self.subscription.write();
            if subscription_slot.is_none() {
                *subscription_slot = Some(SubscriptionConfiguration {
                    inner: Arc::new(SubscriptionConfigurationRef {
                        subscription_manager: self.clone().subscription_manager(runtime),
                        deserializer: Some(Arc::new(SerdeDeserializer)),
                    }),
                });
            }
        }

        SubscriptionBuilder {
            subscription: Some(self.subscription.clone()),
            ..Default::default()
        }
    }

    /// Create subscription listener.
    ///
    /// Listeners configure [`PubNubClient`] to receive real-time updates for
    /// specified list of channels and groups.
    ///
    /// It takes custom runtime which will be used for detached tasks spawning
    /// and delayed task execution.
    /// ```no_run // Starts listening for real-time updates
    /// use futures::StreamExt;
    /// use pubnub::dx::subscribe::{SubscribeStreamEvent, Update};
    /// use pubnub::core::runtime::Runtime;
    /// use std::future::Future;
    ///
    /// #[derive(Clone)]
    /// struct MyRuntime;
    ///
    /// impl Runtime for MyRuntime {
    ///    fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) {
    ///       // spawn the Future
    ///       // e.g. tokio::spawn(future);
    ///    }
    /// }
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
    ///
    /// client
    ///     .subscribe_with_runtime(MyRuntime)
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
    ///
    /// ```
    ///
    /// Instance of [`SubscriptionWithDeserializerBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(not(feature = "serde"))]
    pub fn subscribe_with_runtime<R>(&self, runtime: R) -> SubscriptionWithDeserializerBuilder
    where
        R: Runtime + Send + Sync + 'static,
    {
        {
            // Initialize subscription module when it will be first required.
            let mut subscription_slot = self.subscription.write();
            if subscription_slot.is_none() {
                *subscription_slot = Some(SubscriptionConfiguration {
                    inner: Arc::new(SubscriptionConfigurationRef {
                        subscription_manager: self.clone().subscription_manager(runtime),
                        deserializer: None,
                    }),
                });
            }
        }

        SubscriptionWithDeserializerBuilder {
            subscription: self.subscription.clone(),
        }
    }

    pub(crate) fn subscription_manager<R>(&mut self, runtime: R) -> SubscriptionManager
    where
        R: Runtime + Send + Sync + 'static,
    {
        let channel_bound = 10; // TODO: Think about this value
        let emit_messages_client = self.clone();
        let emit_status_client = self.clone();
        let subscribe_client = self.clone();
        let request_retry_delay_policy = self.config.retry_policy.clone();
        let request_retry_policy = self.config.retry_policy.clone();
        let runtime_sleep = runtime.clone();

        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);

        let engine = EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |params| {
                    let delay_in_secs = request_retry_delay_policy
                        .retry_delay(&params.attempt, params.reason.as_ref());
                    let inner_runtime_sleep = runtime_sleep.clone();

                    Self::subscribe_call(
                        subscribe_client.clone(),
                        params.clone(),
                        Arc::new(move || {
                            if let Some(de) = delay_in_secs {
                                // let rt = inner_runtime_sleep.clone();
                                inner_runtime_sleep.clone().sleep(de).boxed()
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
        );

        SubscriptionManager::new(engine)
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

        let deserializer = {
            let subscription = client
                .subscription
                .read()
                .clone()
                .expect("Subscription configuration is missing");
            subscription
                .deserializer
                .clone()
                .expect("Deserializer is missing")
        };

        let cancel_task = CancellationTask::new(cancel_rx, params.effect_id.to_owned()); // TODO: needs to be owned?

        request
            .execute_with_cancel_and_delay(deserializer, delay, cancel_task)
            .boxed()
    }

    fn emit_status(client: Self, status: &SubscribeStatus) {
        if let Some(manager) = client.subscription.read().as_ref() {
            manager.subscription_manager.notify_new_status(status)
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
            manager.subscription_manager.notify_new_messages(messages)
        }
    }
}

impl<T> PubNubClientInstance<T> {
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
    ///
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(feature = "serde")]
    pub fn subscribe_raw(&self) -> RawSubscriptionBuilder<SerdeDeserializer, T> {
        RawSubscriptionBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(Arc::new(SerdeDeserializer)),
            ..Default::default()
        }
    }

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
    ///
    /// ```
    ///
    /// For more examples see our [examples directory](https://github.com/pubnub/rust/tree/master/examples).
    ///
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(not(feature = "serde"))]
    pub fn subscribe_raw(&self) -> RawSubscriptionWithDeserializerBuilder<T> {
        RawSubscriptionWithDeserializerBuilder {
            client: self.clone(),
        }
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub(crate) fn subscribe_request(&self) -> SubscribeRequestBuilder<T> {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::blocking;
    use crate::{
        core::{TransportRequest, TransportResponse},
        Keyset, PubNubClientBuilder, PubNubGenericClient,
    };

    struct MockTransport;

    #[async_trait::async_trait]
    impl crate::core::Transport for MockTransport {
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

    fn client() -> PubNubGenericClient<MockTransport> {
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
