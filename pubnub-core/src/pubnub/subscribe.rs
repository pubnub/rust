use super::PubNub;
use crate::data::{channel, pubsub};
use crate::runtime::Runtime;
use crate::subscription::Subscription;
use crate::transport::Transport;

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Subscribe to a message stream over the PubNub network.
    ///
    /// The PubNub client only maintains a single subscribe loop for all subscription streams. This
    /// has a benefit that it optimizes for a low number of sockets to the PubNub network. It has a
    /// downside that requires _all_ streams to consume faster than the subscribe loop produces.
    /// A slow consumer will create a head-of-line blocking bottleneck in the processing of
    /// received messages. All streams can only consume as fast as the slowest.
    ///
    /// For example, with 3 total subscription streams and 1 that takes 30 seconds to process each
    /// message; the other 2 streams will be blocked waiting for that 30-second duration on the
    /// slow consumer.
    ///
    /// # Example
    ///
    /// ```
    /// # use pubnub_core::mock::{transport::MockTransport, runtime::MockRuntime};
    /// # let transport = MockTransport::new();
    /// # let runtime = MockRuntime::new();
    /// use futures_util::stream::StreamExt;
    /// use pubnub_core::{data::channel, json::object, Builder};
    ///
    /// # async {
    /// let mut pubnub = Builder::with_components(transport, runtime).build();
    /// let channel_name: channel::Name = "my-channel".parse().unwrap();
    /// let mut stream = pubnub.subscribe(channel_name).await;
    ///
    /// while let Some(message) = stream.next().await {
    ///     println!("Received message: {:?}", message);
    /// }
    /// # };
    /// ```
    pub async fn subscribe(&mut self, channel: channel::Name) -> Subscription<TRuntime> {
        let supervisor_arc_clone = self.subscribe_loop_supervisor.clone();
        let mut supervisor_guard = supervisor_arc_clone.lock().await;
        supervisor_guard
            .subscribe(self, pubsub::SubscribeTo::Channel(channel))
            .await
    }
}
