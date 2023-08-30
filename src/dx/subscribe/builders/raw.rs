//! # PubNub raw subscribe module.
//!
//! This module has all the builders for raw subscription to real-time updates
//! from a list of channels and channel groups.
//!
//! Raw subscription means that subscription will not perform any additional
//! actions than minimal required to receive real-time updates.
//!
//! It is recommended to use [`subscribe`] module instead of this one.
//! [`subscribe`] module has more features and is more user-friendly.
//!
//! This one is used only for special cases when you need to have full control
//! over subscription process or you need more compact subscription solution.

use derive_builder::Builder;

use crate::{
    core::{blocking, Deserializer, PubNubError, Transport},
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{SubscribeCursor, Update},
    },
    lib::alloc::{collections::VecDeque, string::String, string::ToString, vec::Vec},
};

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
#[derive(Builder)]
#[builder(
    pattern = "owned",
    name = "RawSubscriptionBuilder",
    build_fn(private, name = "build_internal", validate = "Self::validate"),
    no_std
)]
pub struct RawSubscription<T, D> {
    /// Current client which can provide transportation to perform the request.
    ///
    /// This field is used to get [`Transport`] to perform the request.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) pubnub_client: PubNubClientInstance<T, D>,

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

    /// `user_id`presence timeout period.
    ///
    /// A heartbeat is a period of time during which `user_id` is visible
    /// `online`.
    /// If, within the heartbeat period, another heartbeat request or a
    /// subscribe (for an implicit heartbeat) request `timeout` will be
    /// announced for `user_id`.
    ///
    /// By default it is set to **300** seconds.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Some(300)"
    )]
    pub(in crate::dx::subscribe) heartbeat: Option<u32>,

    /// Message filtering predicate.
    ///
    /// The [`PubNub`] network can filter out messages published with `meta`
    /// before they reach subscribers using these filtering expressions, which
    /// are based on the definition of the [`filter language`].
    ///
    /// [`PubNub`]:https://www.pubnub.com/
    /// [`filter language`]: https://www.pubnub.com/docs/general/messages/publish#filter-language-definition
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option, into),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,
}

impl<T, D> RawSubscriptionBuilder<T, D> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// subscribe request instance.
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

impl<T, D> RawSubscriptionBuilder<T, D>
where
    T: Transport,
    D: Deserializer,
{
    /// Build [`RawSubscription`] instance.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// [`RawSubscription`] instance.
    ///
    /// It creates a subscription object that can be used to get messages from
    /// PubNub.
    pub fn execute(self) -> Result<RawSubscription<T, D>, PubNubError> {
        self.build_internal()
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl<T, D> RawSubscriptionBuilder<T, D>
where
    T: blocking::Transport,
{
    /// Build [`RawSubscription`] instance.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// [`RawSubscription`] instance.
    ///
    /// It creates a subscription object that can be used to get messages from
    /// PubNub.
    pub fn execute_blocking(self) -> Result<RawSubscription<T, D>, PubNubError> {
        self.build_internal()
            .map_err(|e| PubNubError::SubscribeInitialization {
                details: e.to_string(),
            })
    }
}

impl<T, D> RawSubscription<T, D>
where
    T: Transport + 'static,
    D: Deserializer + 'static,
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
                    .cursor(ctx.cursor.clone())
                    .channels(ctx.subscription.channels.clone())
                    .channel_groups(ctx.subscription.channel_groups.clone());

                if let Some(heartbeat) = ctx.subscription.heartbeat {
                    request = request.heartbeat(heartbeat);
                }

                if let Some(filter_expr) = ctx.subscription.filter_expression.clone() {
                    request = request.filter_expression(filter_expr);
                }

                let response = request.execute().await;

                if let Err(e) = response {
                    return Some((
                        Err(PubNubError::general_api_error(e.to_string(), None, None)),
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

impl<T, D> RawSubscription<T, D>
where
    T: blocking::Transport,
{
    /// Creates subscription iterator.
    ///
    /// This method is used by [`PubNubClient::subscribe_raw`] to build
    /// blocking iterator over messages received from PubNub.
    ///
    /// It loops the subscribe calls and iterator over messages from PubNub.
    pub fn iter(self) -> RawSubscriptionIter<T, D> {
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

impl<T, D> Iterator for RawSubscriptionIter<T, D>
where
    T: blocking::Transport,
    D: Deserializer + 'static,
{
    type Item = Result<Update, PubNubError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ctx = &mut self.0;

        while ctx.messages.is_empty() {
            let mut request = ctx
                .subscription
                .pubnub_client
                .subscribe_request()
                .cursor(ctx.cursor.clone())
                .channels(ctx.subscription.channels.clone())
                .channel_groups(ctx.subscription.channel_groups.clone());

            if let Some(heartbeat) = ctx.subscription.heartbeat {
                request = request.heartbeat(heartbeat);
            }

            if let Some(filter_expr) = ctx.subscription.filter_expression.clone() {
                request = request.filter_expression(filter_expr);
            }

            let response = request.execute_blocking();

            if let Err(e) = response {
                return Some(Err(PubNubError::general_api_error(
                    e.to_string(),
                    None,
                    None,
                )));
            }

            let response = response.expect("Should be Ok");

            let messages: Vec<_> = if let Some(cryptor) = &ctx.subscription.pubnub_client.cryptor {
                response
                    .messages
                    .into_iter()
                    .map(|update| update.decrypt(cryptor))
                    .map(Ok)
                    .collect()
            } else {
                response.messages.into_iter().map(Ok).collect()
            };

            ctx.cursor = response.cursor;
            ctx.messages.extend(messages);
        }

        Some(ctx.messages.pop_front().expect("Shouldn't be empty!"))
    }
}

struct SubscriptionContext<T, D> {
    subscription: RawSubscription<T, D>,
    cursor: SubscribeCursor,
    messages: VecDeque<Result<Update, PubNubError>>,
}

/// Iterator over messages received from PubNub.
///
/// This iterator is returned by [`RawSubscription::iter`] method.
/// It loops the subscribe calls and iterator over messages from PubNub.
/// It can be used to get messages from PubNub.
pub struct RawSubscriptionIter<T, D>(SubscriptionContext<T, D>);

#[cfg(test)]
mod should {
    use super::*;
    use crate::{
        core::{blocking, PubNubError, Transport, TransportRequest, TransportResponse},
        providers::deserialization_serde::DeserializerSerde,
        transport::middleware::PubNubMiddleware,
        Keyset, PubNubClientBuilder,
    };

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

    fn client() -> PubNubClientInstance<PubNubMiddleware<MockTransport>, DeserializerSerde> {
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

    fn sut() -> RawSubscriptionBuilder<PubNubMiddleware<MockTransport>, DeserializerSerde> {
        RawSubscriptionBuilder {
            pubnub_client: Some(client()),
            ..Default::default()
        }
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
