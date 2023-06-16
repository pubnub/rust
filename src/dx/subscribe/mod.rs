//! Subscribe module.
//!
//! Allows subscribe to real-time updates from channels and groups.

pub(crate) mod event_engine;

#[cfg(not(feature = "serde"))]
use crate::core::{Deserialize, Deserializer};
#[cfg(feature = "serde")]
use crate::providers::deserialization_serde::SerdeDeserializer;
use crate::{core::AnyValue, dx::pubnub_client::PubNubClientInstance};

pub use result::SubscribeResponseBody;
pub mod result;

#[doc(inline)]
pub use types::{
    File, Message, MessageAction, Object, Presence, SubscribeCursor, SubscribeMessageType,
    SubscribeStatus,
};
pub mod types;

#[allow(dead_code)]
pub(crate) mod subscription;

#[doc(inline)]
pub use builders::*;

pub mod builders;

impl<T> PubNubClientInstance<T> {
    /// Create subscribe request builder.
    /// This method is used to create events stream for real-time updates on
    /// passed list of channels and groups.
    ///
    /// Instance of [`SubscribeRequestBuilder`] returned.
    #[cfg(feature = "serde")]
    pub fn subscribe<Data>(&self) -> SubscribeRequestBuilder<T, Data, SerdeDeserializer>
    where
        Data: for<'de> serde::Deserialize<'de>,
    {
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
    pub fn subscribe<Data>(&self) -> SubscribeRequestWithDeserializerBuilder<T> {
        SubscribeRequestWithDeserializerBuilder {
            pubnub_client: self.clone(),
        }
    }

    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    pub(in crate::dx::subscribe) async fn handshake(
        &self,
        channels: Option<&[&str]>,
        channel_groups: Option<&[&str]>,
    ) {
        let channel_groups = channel_groups.unwrap_or(&[]);
        let channels = channels.unwrap_or(&[]);
        let _ = self
            .subscribe::<AnyValue>()
            .channels(channels)
            .channel_groups(channel_groups);
        // .execute();
    }

    #[cfg(not(feature = "serde"))]
    #[allow(dead_code)]
    pub(in crate::dx::subscribe) async fn handshake2<D, Data>(
        &self,
        channels: Option<&[&str]>,
        channel_groups: Option<&[&str]>,
        deserializer: D,
    ) where
        Data: for<'data> Deserialize<'data, Data>,
        D: for<'ds> Deserializer<'ds, SubscribeResponseBody<Data>>,
    {
        let channel_groups = channel_groups.unwrap_or(&[]);
        let channels = channels.unwrap_or(&[]);
        let _ = self
            .subscribe::<AnyValue>()
            .deserialize_with(deserializer)
            .channels(channels)
            .channel_groups(channel_groups);
        // .execute();
    }

    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    pub(in crate::dx::subscribe) fn receive(
        &self,
        channels: Option<&[&str]>,
        channel_groups: Option<&[&str]>,
        cursor: SubscribeCursor,
    ) {
        let channel_groups = channel_groups.unwrap_or(&[]);
        let channels = channels.unwrap_or(&[]);
        let _ = self
            .subscribe::<AnyValue>()
            .channels(channels)
            .channel_groups(channel_groups)
            .cursor(cursor);
        // .execute();
    }

    #[cfg(not(feature = "serde"))]
    #[allow(dead_code)]
    pub(in crate::dx::subscribe) fn receive<D, Data>(
        &self,
        channels: Option<&[&str]>,
        channel_groups: Option<&[&str]>,
        cursor: SubscribeCursor,
        deserializer: D,
    ) where
        Data: for<'data> Deserialize<'data, Data>,
        D: for<'ds> Deserializer<'ds, SubscribeResponseBody<Data>>,
    {
        let channel_groups = channel_groups.unwrap_or(&[]);
        let channels = channels.unwrap_or(&[]);
        let _ = self
            .subscribe::<AnyValue>()
            .deserialize_with(deserializer)
            .channels(channels)
            .channel_groups(channel_groups)
            .cursor(cursor);
        // .execute();
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::AnyValue;
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
        let _ = client().subscribe::<String>();
    }

    #[tokio::test]
    async fn make_handshake() {
        let response = client()
            .subscribe::<AnyValue>()
            .channels(&["hello", "world"])
            .execute()
            .await;

        assert!(response.is_ok(), "Request should success");
        if let Ok(result) = response {
            assert_ne!(result.cursor.timetoken, "0");
            assert_ne!(result.cursor.region, 0);
            assert_eq!(result.messages.len(), 0);
        } else {
            panic!("Handshake request did fail.");
        }
    }
}
