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
};
pub mod types;

use crate::{
    core::{event_engine::EventEngine, PubNubError, Transport},
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::result::{SubscribeResult, Update},
    },
    lib::alloc::{boxed::Box, string::String, sync::Arc, vec::Vec},
};

pub(crate) use subscription_manager::SubscriptionManager;
pub(crate) mod subscription_manager;

#[doc(inline)]
pub use builders::*;
pub mod builders;

impl<T> PubNubClientInstance<T>
where
    T: Transport + Send + 'static,
{
    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub fn subscribe(&mut self) -> SubscriptionBuilder {
        {
            // Initialize manager when it will be first required.
            let mut manager_slot = self.subscription_manager.write();
            if manager_slot.is_none() {
                *manager_slot = Some(self.clone().subscription_manager());
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

    pub(crate) fn subscription_manager(&mut self) -> SubscriptionManager {
        let handshake_client = self.clone();
        let receive_client = self.clone();

        let engine = EventEngine::new(
            SubscribeEffectHandler::new(
                Arc::new(move |channels, channel_groups, attempt, reason| {
                    Self::handshake(
                        handshake_client.clone(),
                        SubscriptionParams {
                            channels,
                            channel_groups,
                            _attempt: attempt,
                            _reason: reason,
                        },
                    )
                }),
                Arc::new(move |channels, channel_groups, cursor, attempt, reason| {
                    Self::receive(
                        receive_client.clone(),
                        cursor,
                        SubscriptionParams {
                            channels,
                            channel_groups,
                            _attempt: attempt,
                            _reason: reason,
                        },
                    )
                }),
                Arc::new(|| {
                    // Do nothing yet
                }),
                Arc::new(Box::new(|| {
                    // Do nothing yet
                })),
            ),
            SubscribeState::Unsubscribed,
        );

        // self.subscription_manager = Some(Arc::new(RwLock::new(SubscriptionManager::new(engine))));
        SubscriptionManager::new(engine)
    }

    #[allow(dead_code)]
    pub(crate) fn handshake(
        client: Self,
        params: SubscriptionParams,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
        // TODO: Add retry policy check and error if failed.
        Self::receive(client, &SubscribeCursor::default(), params)
    }

    #[allow(dead_code)]
    pub(crate) fn receive(
        client: Self,
        cursor: &SubscribeCursor,
        params: SubscriptionParams,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
        // TODO: Add retry policy check and error if failed.
        let mut request = client.subscribe_request().cursor(cursor.clone());

        if let Some(channels) = params.channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = params.channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }

        request.execute().boxed()
    }

    #[allow(dead_code)]
    fn emit_status(&self, status: &SubscribeStatus) {
        self.subscription_manager
            .read()
            .as_ref()
            .map(|manager| manager.notify_new_status(status));
    }

    #[allow(dead_code)]
    fn emit_messages(&self, messages: Vec<Update>) {
        self.subscription_manager
            .read()
            .as_ref()
            .map(|manager| manager.notify_new_messages(messages));
    }
}

pub(crate) struct SubscriptionParams<'execution> {
    channels: &'execution Option<Vec<String>>,
    channel_groups: &'execution Option<Vec<String>>,
    _attempt: u8,
    _reason: Option<PubNubError>,
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

    #[test]
    fn create_builder() {
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
}
