use crate::dx::subscribe::types::SubscribeStreamEvent;
use crate::{
    core::PubNubError,
    dx::{
        pubnub_client::PubNubClientInstance,
        subscribe::{builders, result::Update, SubscribeCursor},
    },
    lib::alloc::{string::String, vec::Vec},
};
use derive_builder::Builder;
use futures::{
    task::{Context, Poll},
    Stream,
};
use phantom_type::PhantomType;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

/// Subscription that is responsible for getting messages from PubNub.
///
/// Subscription provides a way to get messages from PubNub. It is responsible
/// for handshake and receiving messages.
///
#[derive(Builder, Debug)]
#[builder(
    pattern = "owned",
    build_fn(private, name = "build_internal", validate = "Self::validate"),
    no_std
)]
pub struct Subscription<Transport> {
    /// Current client which can provide transportation to perform the request.
    #[builder(field(vis = "pub(in crate::dx::subscribe)"), setter(custom))]
    pub(in crate::dx::subscribe) pubnub_client: PubNubClientInstance<Transport>,

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
    pub(in crate::dx::subscribe) cursor: Option<SubscribeCursor>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "Some(300)"
    )]
    pub(in crate::dx::subscribe) heartbeat: Option<u32>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(strip_option),
        default = "None"
    )]
    pub(in crate::dx::subscribe) filter_expression: Option<String>,

    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        setter(custom),
        default = "Uuid::new_v4().to_string()"
    )]
    pub(in crate::dx::subscribe) id: String,

    /// Used transport type.
    ///
    /// Type which should be used as transport for network requests.
    #[builder(
        field(vis = "pub(in crate::dx::subscribe)"),
        default = "Default::default()"
    )]
    pub(in crate::dx::subscribe) _transport: PhantomType<Transport>,
}

impl<Transport> SubscriptionBuilder<Transport> {
    /// Validate user-provided data for request builder.
    ///
    /// Validator ensure that list of provided data is enough to build valid
    /// request instance.
    fn validate(&self) -> Result<(), String> {
        let groups_len = self.channel_groups.as_ref().map_or_else(|| 0, |v| v.len());
        let channels_len = self.channels.as_ref().map_or_else(|| 0, |v| v.len());

        builders::validate_configuration(&self.pubnub_client).and_then(|_| {
            if channels_len == groups_len && channels_len == 0 {
                Err("Either channels or channel groups should be provided".into())
            } else {
                Ok(())
            }
        })
    }
}

impl<Transport> SubscriptionBuilder<Transport> {
    /// Construct subscription object.
    pub fn build(self) -> Result<Arc<Subscription<Transport>>, SubscriptionBuilderError> {
        self.build_internal()
            .map(|subscription| Arc::new(subscription))
            .map(|subscription| {
                subscription
                    .pubnub_client
                    .subscription_manager
                    .as_ref()
                    .map(|manager| manager.clone().write().register(subscription.clone()));
                subscription
            })
    }
}

impl<Transport> Stream for Subscription<Transport> {
    type Item = Result<Vec<Update>, PubNubError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

impl<Transport> Subscription<Transport> {
    pub(crate) fn notify_update(&self, update: SubscribeStreamEvent) {
        println!("Report listeners about new update: {update:?}")
    }
}

//
// impl Subscription {
//     pub(crate) fn subscribe() -> Self {
//         // // TODO: implementation is a part of the different task
//         // let handshake: HandshakeFunction = |&_, &_, _, _| Ok(vec![]);
//         // let receive: ReceiveFunction = |&_, &_, &_, _, _| Ok(vec![]);
//         //
//         // Self {
//         //     engine: SubscribeEngine::new(
//         //         SubscribeEffectHandler::new(handshake, receive),
//         //         SubscribeState::Unsubscribed,
//         //     ),
//         // }
//         Self { /* fields */ }
//     }
// }
