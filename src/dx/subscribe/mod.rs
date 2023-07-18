//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

pub(crate) mod event_engine;
use event_engine::{SubscribeEffectHandler, SubscribeState};

use futures::{future::BoxFuture, FutureExt};

#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;

pub use result::SubscribeResponseBody;
pub mod result;

#[doc(inline)]
pub use types::{
    File, MessageAction, Object, Presence, SubscribeCursor, SubscribeMessageType, SubscribeStatus,
    SubscribeStreamEvent,
};
pub mod types;

use crate::{
    core::{event_engine::EventEngine, runtime::Runtime, PubNubError, Transport},
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::result::{SubscribeResult, Update},
    },
    lib::alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc, vec::Vec},
};

pub(crate) use subscription_manager::SubscriptionManager;
pub(crate) mod subscription_manager;

#[doc(inline)]
pub use builders::*;
pub mod builders;

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
    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    #[cfg(feature = "tokio")]
    pub fn subscribe(&self) -> SubscriptionBuilder {
        use crate::providers::futures_tokio::TokioRuntime;

        self.subscribe_with_runtime(TokioRuntime)
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// It takes custom runtime which will be used for detached tasks spawning
    /// and delayed task execution.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub fn subscribe_with_runtime<R>(&self, runtime: R) -> SubscriptionBuilder
    where
        R: Runtime + 'static,
    {
        {
            // Initialize manager when it will be first required.
            let mut manager_slot = self.subscription_manager.write();
            if manager_slot.is_none() {
                *manager_slot = Some(self.clone().subscription_manager(runtime.clone()));
            }
        }

        SubscriptionBuilder {
            subscription_manager: Some(self.subscription_manager.clone()),
            ..Default::default()
        }
    }

    // /// Create subscribe request builder.
    // /// This method is used to create events stream for real-time updates on
    // /// passed list of channels and groups.
    // ///
    // /// Instance of [`SubscribeRequestBuilder`] returned.
    // #[cfg(not(feature = "serde"))]
    // pub fn subscribe(&self) -> SubscribeRequestWithDeserializerBuilder<T> {
    //     SubscribeRequestWithDeserializerBuilder {
    //         pubnub_client: self.clone(),
    //     }
    // }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    #[cfg(feature = "serde")]
    pub(crate) fn subscribe_request(&self) -> SubscribeRequestBuilder<T, SerdeDeserializer> {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(SerdeDeserializer),
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

        let cancel_task = CancellationTask::new(cancel_rx, params.effect_id.to_owned()); // TODO: needs to be owned?

        request.execute(cancel_task).boxed()
    }

    fn emit_status(client: Self, status: &SubscribeStatus) {
        client
            .subscription_manager
            .read()
            .as_ref()
            .map(|manager| manager.notify_new_status(status));
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

        client
            .subscription_manager
            .read()
            .as_ref()
            .map(|manager| manager.notify_new_messages(messages));
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
    async fn create_builder() {
        let _ = client().subscribe();
    }

    #[tokio::test]
    async fn make_handshake() {
        let _subscription = client()
            .subscribe()
            .channels(["hello".into(), "world".into()].to_vec())
            .build();

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
