//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

pub(crate) mod event_engine;
use event_engine::{SubscribeEffectHandler, SubscribeState};

use futures::{future::BoxFuture, FutureExt};

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

use crate::{
    core::{event_engine::EventEngine, runtime::Runtime, PubNubError, Transport},
    dx::{pubnub_client::PubNubClientInstance, subscribe::result::SubscribeResult},
    lib::alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc, vec::Vec},
};

pub(crate) use subscription_manager::SubscriptionManager;
pub(crate) mod subscription_manager;

pub(crate) use subscription_configuration::{
    SubscriptionConfiguration, SubscriptionConfigurationRef,
};
pub(crate) mod subscription_configuration;

#[doc(inline)]
pub use builders::*;
pub mod builders;

#[doc(inline)]
use cancel::CancellationTask;
mod cancel;

#[allow(dead_code)]
pub(crate) struct SubscriptionParams<'execution> {
    channels: &'execution Option<Vec<String>>,
    channel_groups: &'execution Option<Vec<String>>,
    cursor: Option<&'execution SubscribeCursor>,
    attempt: u8,
    reason: Option<PubNubError>,
    effect_id: &'execution str,
}

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
    ///    client
    ///        .subscribe()
    ///        .channels(["hello".into(), "world".into()].to_vec())
    ///        .execute()?
    ///        .stream()
    ///        .for_each(|event| async move {
    ///            match event {
    ///                SubscribeStreamEvent::Update(update) => println!("update: {:?}", update),
    ///                SubscribeStreamEvent::Status(status) => println!("status: {:?}", status),
    ///            }
    ///        })
    ///        .await;
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
    /// Instance of [`SubscriptionBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(feature = "serde")]
    pub fn subscribe_with_runtime<R>(&self, runtime: R) -> SubscriptionBuilder
    where
        R: Runtime + 'static,
    {
        {
            // Initialize subscription module when it will be first required.
            let mut subscription_slot = self.subscription.write();
            if subscription_slot.is_none() {
                *subscription_slot = Some(SubscriptionConfiguration {
                    inner: Arc::new(SubscriptionConfigurationRef {
                        subscription_manager: Arc::new(self.clone().subscription_manager(runtime)),
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
    ///
    /// Instance of [`SubscriptionWithDeserializerBuilder`] returned.
    /// [`PubNubClient`]: crate::PubNubClient
    #[cfg(not(feature = "serde"))]
    pub fn subscribe_with_runtime<R>(&self, runtime: R) -> SubscriptionWithDeserializerBuilder
    where
        R: Runtime + 'static,
    {
        {
            // Initialize subscription module when it will be first required.
            let mut subscription_slot = self.subscription.write();
            if subscription_slot.is_none() {
                *subscription_slot = Some(SubscriptionConfiguration {
                    inner: Arc::new(SubscriptionConfigurationRef {
                        subscription_manager: Arc::new(self.clone().subscription_manager(runtime)),
                        deserializer: None,
                    }),
                });
            }
        }

        SubscriptionWithDeserializerBuilder {
            subscription: self.subscription.clone(),
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

    pub(crate) fn subscription_manager<R>(&mut self, runtime: R) -> SubscriptionManager
    where
        R: Runtime + 'static,
    {
        let channel_bound = 10; // TODO: Think about this value
        let emit_messages_client = self.clone();
        let emit_status_client = self.clone();
        let subscribe_client = self.clone();
        let retry_policy = self.config.clone().retry_policy;

        let (cancel_tx, cancel_rx) = async_channel::bounded::<String>(channel_bound);

        let engine = EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |params| {
                    Self::subscribe_call(subscribe_client.clone(), cancel_rx.clone(), params)
                }),
                Arc::new(move |status| Self::emit_status(emit_status_client.clone(), &status)),
                Arc::new(Box::new(move |updates| {
                    Self::emit_messages(emit_messages_client.clone(), updates)
                })),
                retry_policy,
                cancel_tx,
            ),
            SubscribeState::Unsubscribed,
            runtime,
        );

        SubscriptionManager::new(engine)
    }

    #[allow(dead_code)]
    pub(crate) fn subscribe_call(
        client: Self,
        cancel_rx: async_channel::Receiver<String>,
        params: SubscriptionParams,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
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

        request.execute(deserializer, cancel_task).boxed()
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

#[cfg(feature = "blocking")]
impl<T> PubNubClientInstance<T> where T: crate::core::blocking::Transport {}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        transport::{middleware::PubNubMiddleware, TransportReqwest},
        Keyset, PubNubClientBuilder,
    };

    #[allow(dead_code)]
    fn client() -> PubNubClientInstance<PubNubMiddleware<TransportReqwest>> {
        PubNubClientBuilder::with_reqwest_transport()
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
    #[cfg(feature = "std")]
    async fn create_builder() {
        let _ = client().subscribe();
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn make_handshake() {
        let _subscription = client()
            .subscribe()
            .channels(["hello".into(), "world".into()].to_vec())
            .execute();

        // if let Ok(subscription) = subscription {
        //     subscription
        //         .for_each(|data| async {
        //             match data {
        //                 Ok(update) => println!("~~~> Update: {:?}", update),
        //                 Err(err) => println!("~~~> Error:P{}", err),
        //             };
        //         })
        //         .await
        // }
        // let response = client()
        //     .subscribe()
        //     .channels(["hello".into(), "world".into()].to_vec())
        //     .execute()
        //     .await;
        //
        // assert!(response.is_ok(), "Request should success");
        // if let Ok(result) = response {
        //     assert_ne!(result.cursor.timetoken, "0");
        //     assert_ne!(result.cursor.region, 0);
        //     assert_eq!(result.messages.len(), 0);
        // } else {
        //     panic!("Handshake request did fail.");
        // }
    }

    // TODO: add possibility to cancel subscription
    //    #[tokio::test]
    //    async fn cancel_effect() {
    //        let mut client = client();
    //
    //        let subscription = client
    //            .subscribe()
    //            .channels(["hello".into(), "world".into()].to_vec())
    //            .build()
    //            .unwrap();
    //
    //        subscription.cancel().await;
    //
    //        let error = subscription.stream().await.unwrap_err();
    //
    //        assert!(matches!(error, PubNubError::EffectCanceled));
    //    }
}
