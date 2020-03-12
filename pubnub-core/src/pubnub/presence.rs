use super::PubNub;
use crate::data::channel;
use crate::runtime::Runtime;
use crate::subscription::Subscription;
use crate::transport::Transport;

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Subscribe to presence events for the specified channel.
    ///
    /// This is just a tiny wrapper that calls [`PubNub::subscribe`]
    /// internally with the specified channel name with a `-pnpres` suffix.
    pub async fn subscribe_to_presence(
        &mut self,
        channel: channel::Name,
    ) -> Subscription<TRuntime> {
        let channel = channel::Name::from_string_unchecked(format!("{}-pnpres", channel));
        self.subscribe(channel).await
    }
}
