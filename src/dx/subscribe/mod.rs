//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

pub(crate) mod event_engine;

use event_engine::{SubscribeEffectHandler, SubscribeState};
use futures::future::BoxFuture;
use spin::rwlock::RwLock;
use std::future::Future;

#[cfg(not(feature = "serde"))]
use crate::core::{Deserialize, Deserializer};
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
    core::{event_engine::EventEngine, PubNubError},
    dx::{pubnub_client::PubNubClientInstance, subscribe::result::SubscribeResult},
    lib::core::sync::Arc,
};

pub(crate) mod subscription_manager;

use crate::dx::subscribe::result::Update;
use crate::dx::subscribe::subscription_manager::SubscriptionManager;
#[doc(inline)]
pub use builders::*;

pub mod builders;

impl<Transport> PubNubClientInstance<Transport> {
    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    pub fn subscribe(&self) -> SubscriptionBuilder<Transport> {
        SubscriptionBuilder {
            pubnub_client: Some(self.clone()),
            ..Default::default()
        }
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    #[cfg(feature = "serde")]
    pub(crate) fn subscribe_request(
        &self,
    ) -> SubscribeRequestBuilder<Transport, SerdeDeserializer> {
        SubscribeRequestBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(SerdeDeserializer),
            ..Default::default()
        }
    }

    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    #[cfg(not(feature = "serde"))]
    pub fn subscribe(&self) -> SubscribeRequestWithDeserializerBuilder<Transport> {
        SubscribeRequestWithDeserializerBuilder {
            pubnub_client: self.clone(),
        }
    }

    #[cfg(feature = "subscribe")]
    pub(crate) fn setup_event_engines(&mut self) {
        let engine = EventEngine::new(
            SubscribeEffectHandler::new(
                self.clone(),
                Self::handshake,
                Self::receive,
                Self::emit_status,
                Self::emit_messages,
            ),
            SubscribeState::Unsubscribed,
        );

        self.subscription_manager = Some(Arc::new(RwLock::new(SubscriptionManager::new(engine))));
    }

    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    fn handshake(
        client: PubNubClientInstance<Transport>,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
        attempt: u8,
        reason: Option<PubNubError>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
        // TODO: Add retry policy check and error if failed.
        Self::receive(
            client,
            channels,
            channel_groups,
            &SubscribeCursor::default(),
            attempt,
            reason,
        )
    }

    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    fn receive(
        client: PubNubClientInstance<Transport>,
        channels: &Option<Vec<String>>,
        channel_groups: &Option<Vec<String>>,
        cursor: &SubscribeCursor,
        attempt: u8,
        reason: Option<PubNubError>,
    ) -> BoxFuture<'static, Result<SubscribeResult, PubNubError>> {
        // TODO: Add retry policy check and error if failed.
        let mut request = client.subscribe_request().cursor(cursor.clone());

        if let Some(channels) = channels.clone() {
            request = request.channels(channels);
        }

        if let Some(channel_groups) = channel_groups.clone() {
            request = request.channel_groups(channel_groups);
        }

        // BoxFuture::new(request.build())
        Box::pin(request.exec_second())
    }

    #[allow(dead_code)]
    fn emit_status(client: PubNubClientInstance<Transport>, status: &SubscribeStatus) {
        client
            .subscription_manager
            .map(|manager| manager.read().notify_new_status(status));
    }

    #[allow(dead_code)]
    fn emit_messages(client: PubNubClientInstance<Transport>, messages: Vec<Update>) {
        client
            .subscription_manager
            .map(|manager| manager.read().notify_new_messages(messages));
    }

    // #[cfg(feature = "serde")]
    // #[allow(dead_code)]
    // pub(in crate::dx::subscribe) fn receive2<Data>(
    //     &self,
    //     channels: &Option<Vec<String>>,
    //     channel_groups: &Option<Vec<String>>,
    //     cursor: &SubscribeCursor,
    //     attempt: u8,
    //     reason: Option<PubNubError>,
    // ) -> Result<Vec<SubscribeEvent>, PubNubError> {
    //     // TODO: Add retry policy check and error if failed.
    //     let mut request = self.subscribe::<Data>().cursor(cursor.clone());
    //
    //     if let Some(channels) = channels.clone() {
    //         request = request.channels(channels);
    //     }
    //
    //     if let Some(channel_groups) = channel_groups.clone() {
    //         request = request.channel_groups(channel_groups);
    //     }
    //
    //     Ok(Vec::new())
    // }

    //
    // #[cfg(not(feature = "serde"))]
    // #[allow(dead_code)]
    // pub(in crate::dx::subscribe) fn receive<D, Data>(
    //     &self,
    //     channels: Option<&[&str]>,
    //     channel_groups: Option<&[&str]>,
    //     cursor: SubscribeCursor,
    //     deserializer: D,
    // ) where
    //     Data: for<'data> Deserialize<'data, Data>,
    //     D: for<'ds> Deserializer<'ds, SubscribeResponseBody<Data>>,
    // {
    //     let channel_groups = channel_groups.unwrap_or(&[]);
    //     let channels = channels.unwrap_or(&[]);
    //     let _ = self
    //         .subscribe::<AnyValue>()
    //         .deserialize_with(deserializer)
    //         .channels(channels)
    //         .channel_groups(channel_groups)
    //         .cursor(cursor);
    //     // .execute();
    // }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        transport::{middleware::PubNubMiddleware, TransportReqwest},
        Keyset, PubNubClientBuilder,
    };
    use futures::stream::StreamExt;

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
        let subscription = client()
            .subscribe()
            .channels(["hello".into(), "world".into()].to_vec())
            .build();

        if let Ok(subscription) = subscription {
            subscription
                .for_each(|data| async {
                    match data {
                        Ok(update) => println!("~~~> Update: {:?}", update),
                        Err(err) => println!("~~~> Error:P{}", err),
                    };
                })
                .await
        }
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
