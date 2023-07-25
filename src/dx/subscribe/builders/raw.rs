//! # PubNub raw subscribe module.
//!
//! This module has all the builders for raw subscription to real-time updates from
//! a list of channels and channel groups.
//!
//! Raw subscription means that subscription will not perform any additional
//! actions than minimal required to receive real-time updates.
//!
//! It is recommended to use [`subscribe`] module instead of this one.
//! [`subscribe`] module has more features and is more user-friendly.
//!
//! This one is used only for special cases when you need to have full control
//! over subscription process or you need more compact subscription solution.

use crate::dx::subscribe::Update;
use crate::lib::alloc::{collections::VecDeque, sync::Arc};
use crate::{
    core::{blocking, Deserializer, PubNubError, Transport},
    dx::subscribe::{
        SubscribeCursor, SubscribeResponseBody,
    },
    PubNubGenericClient,
};
use derive_builder::Builder;
use uuid::Uuid;

/// Raw subscription that is responsible for getting messages from PubNub.
///
/// In difference from [`Subscription`] this one is not responsible for
/// maintaining subscription loop and does not have any additional features.
/// It makes simple requests to PubNub and returns received messages.
///
/// It is recommended to use [`Subscription`] instead of this one.
/// [`Subscription`] has more features and is more user-friendly.
///
/// It should not be created directly, but via [`PubNubClient::subscribe`]
/// and wrapped in [`Subscription`] struct.
#[derive(Debug, Builder)]
#[builder(
    pattern = "owned",
    name = "RawSubscriptionBuilder",
    build_fn(private, name = "build_internal", validate = "Self::validate"),
    no_std
)]
pub struct RawSubscriptionRef<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
{
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) pubnub_client: PubNubGenericClient<T>,

    /// Channels from which real-time updates should be received.
    ///
    /// List of channels on which [`PubNubClient`] will subscribe and notify
    /// about received real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(into, strip_option),
        default = "Vec::new()"
    )]
    pub(in crate::dx::subscribe) channels: Vec<String>,

    /// Channel groups from which real-time updates should be received.
    ///
    /// List of groups of channels on which [`PubNubClient`] will subscribe and
    /// notify about received real-time updates.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(into, strip_option),
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
    pub(in crate::dx::subscribe) cursor: Option<u64>,

    /// Deserializer.
    ///
    /// Deserializer which will be used to deserialize received messages
    /// from PubNub.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) deserializer: Arc<D>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Some(300)"
    )]
    pub(in crate::dx::subscribe) heartbeat: Option<u32>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option, into),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "Uuid::new_v4().to_string()"
    )]
    pub(in crate::dx::subscribe) id: String,
}

impl<D, T> RawSubscriptionBuilder<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
{
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        if channels_len == groups_len && channels_len == 0 {
            Err("Either channels or channel groups should be provided".into())
        } else {
            Ok(())
        }
    }
}

impl<D, T> RawSubscriptionBuilder<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
    T: Transport,
{
    /// Build [`RawSubscription`] instance.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// [`RawSubscription`] instance.
    ///
    /// It creates a subscription object that can be used to get messages from
    /// PubNub.
    fn execute_blocking(self) -> Result<RawSubscriptionRef<D, T>, PubNubError> {
        self.build_internal()
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl<D, T> RawSubscriptionBuilder<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
    T: blocking::Transport,
{
    /// Build [`RawSubscription`] instance.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// [`RawSubscription`] instance.
    ///
    /// It creates a subscription object that can be used to get messages from
    /// PubNub.
    fn execute(self) -> Result<RawSubscriptionRef<D, T>, PubNubError> {
        self.build_internal()
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl<D, T> RawSubscriptionRef<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
    T: Transport + 'static,
{
    /// Creates subscription stream.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// stream over messages received from PubNub.
    ///
    /// It creates a stream that can be awaited to get messages from PubNub.
    pub fn stream(self) -> impl futures::Stream<Item = Result<Update, PubNubError>> {
        let cursor = self
            .cursor
            .map(|tt| SubscribeCursor {
                timetoken: tt.to_string(),
                region: 0,
            })
            .unwrap_or_default();

        let context = SubscriptionContext {
            subscription: self,
            cursor,
            messages: VecDeque::new(),
        };

        futures::stream::unfold(context, |mut ctx| async {
            while ctx.messages.is_empty() {
                let mut request = ctx
                    .subscription
                    .pubnub_client
                    .subscribe_request()
                    .cursor(ctx.cursor.clone());

                request = request.channels(ctx.subscription.channels.clone());

                request = request.channel_groups(ctx.subscription.channel_groups.clone());

                let deserializer = ctx.subscription.deserializer.clone();

                let response = request.execute(deserializer).await;

                if let Err(e) = response {
                    return Some((
                        Err(PubNubError::general_api_error(e.to_string(), None)),
                        ctx,
                    ));
                }

                let response = response.expect("Should be Ok");

                ctx.cursor = response.cursor;
                ctx.messages.extend(response.messages.into_iter().map(Ok));
            }

            Some((ctx.messages.pop_front().expect("Shouldn't be empty!"), ctx))
        })
    }
}

impl<D, T> RawSubscriptionRef<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
    T: blocking::Transport,
{
    /// Creates subscription iterator.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// blocking iterator over messages received from PubNub.
    ///
    /// It loops the subscribe calls and iterator over messages from PubNub.
    pub fn iter(self) -> RawSubscriptionIter<D, T> {
        let cursor = self
            .cursor
            .map(|tt| SubscribeCursor {
                timetoken: tt.to_string(),
                region: 0,
            })
            .unwrap_or_default();

        let context = SubscriptionContext {
            subscription: self,
            cursor,
            messages: VecDeque::new(),
        };

        RawSubscriptionIter(context)
    }
}

impl<D, T> Iterator for RawSubscriptionIter<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
    T: blocking::Transport,
{
    type Item = Result<Update, PubNubError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ctx = &mut self.0;

        while ctx.messages.is_empty() {
            let mut request = ctx
                .subscription
                .pubnub_client
                .subscribe_request()
                .cursor(ctx.cursor.clone());

            request = request.channels(ctx.subscription.channels.clone());

            request = request.channel_groups(ctx.subscription.channel_groups.clone());

            let deserializer = ctx.subscription.deserializer.clone();

            let response = request.execute_blocking(deserializer);

            if let Err(e) = response {
                return Some(Err(PubNubError::general_api_error(e.to_string(), None)));
            }

            let response = response.expect("Should be Ok");

            ctx.cursor = response.cursor;
            ctx.messages.extend(response.messages.into_iter().map(Ok));
        }

        Some(ctx.messages.pop_front().expect("Shouldn't be empty!"))
    }
}

struct SubscriptionContext<D, T>
where
    D: Deserializer<SubscribeResponseBody>,
{
    subscription: RawSubscriptionRef<D, T>,
    cursor: SubscribeCursor,
    messages: VecDeque<Result<Update, PubNubError>>,
}

pub struct RawSubscriptionIter<D, T>(SubscriptionContext<D, T>)
where
    D: Deserializer<SubscribeResponseBody>;

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::{
            blocking, Deserializer, PubNubError, Transport, TransportRequest, TransportResponse,
        },
        dx::subscribe::{result::APISuccessBody, SubscribeResponseBody},
        Keyset, PubNubClientBuilder, PubNubGenericClient,
    };

    struct MockDeserializer;

    impl Deserializer<SubscribeResponseBody> for MockDeserializer {
        fn deserialize(&self, _body: &[u8]) -> Result<SubscribeResponseBody, PubNubError> {
            Ok(SubscribeResponseBody::SuccessResponse(APISuccessBody {
                cursor: Default::default(),
                messages: Default::default(),
            }))
        }
    }

    struct MockTransport;

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        async fn send(&self, _req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            // Send your request here

            Ok(TransportResponse::default())
        }
    }

    impl blocking::Transport for MockTransport {
        fn send(&self, _req: TransportRequest) -> Result<TransportResponse, PubNubError> {
            // Send your request here

            Ok(TransportResponse::default())
        }
    }

    fn client() -> PubNubGenericClient<MockTransport> {
        PubNubClientBuilder::with_transport(MockTransport)
            .with_keyset(Keyset {
                subscribe_key: "demo",
                publish_key: None,
                secret_key: None,
            })
            .with_user_id("rust-test-user")
            .build()
            .unwrap()
    }

    fn sut() -> RawSubscriptionBuilder<MockDeserializer, MockTransport> {
        let mut sut = RawSubscriptionBuilder::<MockDeserializer, MockTransport>::default();

        sut.pubnub_client = Some(client());
        sut.deserializer = Some(Arc::new(MockDeserializer));

        sut
    }

    #[test]
    fn validate_channels_and_channel_groups() {
        let builder = sut();
        assert!(builder.validate().is_err());

        let builder = sut().channels(vec!["ch1".into()]);
        assert!(builder.validate().is_ok());

        let builder = sut().channel_groups(vec!["cg1".into()]);
        assert!(builder.validate().is_ok());
    }

    #[tokio::test]
    async fn call_subscribe_endpoint_async() {
        use futures::StreamExt;
        let message = sut()
            .channels(vec!["ch1".into()])
            .execute()
            .unwrap()
            .stream()
            .boxed()
            .next()
            .await;

        assert!(message.is_some());
    }

    #[test]
    fn call_subscribe_endpoint_blocking() {
        let message = sut()
            .channels(vec!["ch1".into()])
            .execute_blocking()
            .unwrap()
            .iter()
            .next();

        assert!(message.is_some());
    }
}
