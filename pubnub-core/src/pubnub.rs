use crate::data::object::Object;
use crate::data::request;
use crate::data::timetoken::Timetoken;
use crate::runtime::Runtime;
use crate::subscription::subscribe_loop_supervisor::SubscribeLoopSupervisor;
use crate::subscription::Subscription;
use crate::transport::Transport;
use futures_util::lock::Mutex;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// # PubNub Client
///
/// The PubNub lib implements socket pools to relay data requests as a client
/// connection to the PubNub Network.
#[derive(Clone, Debug)]
pub struct PubNub<TTransport, TRuntime>
where
    TTransport: Transport,
    TRuntime: Runtime,
{
    /// Transport to use for communication.
    pub(crate) transport: TTransport,
    /// Runtime to use for managing resources.
    pub(crate) runtime: TRuntime,

    /// Subscribe loop lifecycle management.
    pub(crate) subscribe_loop_supervisor: Arc<Mutex<SubscribeLoopSupervisor>>,
}

impl<TTransport, TRuntime> PubNub<TTransport, TRuntime>
where
    TTransport: Transport + 'static,
    TRuntime: Runtime + 'static,
{
    /// Publish a message over the PubNub network.
    ///
    /// # Example
    ///
    /// ```
    /// # use pubnub_core::mock::{transport::MockTransport, runtime::MockRuntime};
    /// # let transport = MockTransport::new();
    /// # let runtime = MockRuntime::new();
    /// use pubnub_core::{json::object, Builder};
    ///
    /// # async {
    /// let pubnub = Builder::with_components(transport, runtime).build();
    ///
    /// let timetoken = pubnub
    ///     .publish(
    ///         "my-channel",
    ///         object! {
    ///             "username" => "JoeBob",
    ///             "content" => "Hello, world!",
    ///         },
    ///     )
    ///     .await?;
    ///
    /// println!("Timetoken: {}", timetoken);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub async fn publish(
        &self,
        channel: &str,
        message: Object,
    ) -> Result<Timetoken, TTransport::Error> {
        let request = request::PublishV1 {
            channel: channel.to_string(),
            meta: None,
            payload: message,
        };
        self.transport.publish_request_v1(request).await
    }

    /// Publish a message over the PubNub network with an extra metadata payload.
    ///
    /// # Example
    ///
    /// ```
    /// # use pubnub_core::mock::{transport::MockTransport, runtime::MockRuntime};
    /// # let transport = MockTransport::new();
    /// # let runtime = MockRuntime::new();
    /// use pubnub_core::{json::object, Builder};
    ///
    /// # async {
    /// let pubnub = Builder::with_components(transport, runtime).build();
    ///
    /// let message = object! {
    ///     "username" => "JoeBob",
    ///     "content" => "Hello, world!",
    /// };
    /// let metadata = object! {
    ///     "uuid" => "JoeBob",
    /// };
    ///
    /// let timetoken = pubnub
    ///     .publish_with_metadata("my-channel", message, metadata)
    ///     .await?;
    ///
    /// println!("Timetoken: {}", timetoken);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub async fn publish_with_metadata(
        &self,
        channel: &str,
        message: Object,
        metadata: Object,
    ) -> Result<Timetoken, TTransport::Error> {
        let request = request::PublishV1 {
            channel: channel.to_string(),
            meta: Some(metadata),
            payload: message,
        };
        self.transport.publish_request_v1(request).await
    }

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
    /// use pubnub_core::{json::object, Builder};
    ///
    /// # async {
    /// let mut pubnub = Builder::with_components(transport, runtime).build();
    /// let mut stream = pubnub.subscribe("my-channel").await;
    ///
    /// while let Some(message) = stream.next().await {
    ///     println!("Received message: {:?}", message);
    /// }
    /// # };
    /// ```
    pub async fn subscribe(&mut self, channel: &str) -> Subscription<TRuntime> {
        let supervisor_arc_clone = self.subscribe_loop_supervisor.clone();
        let mut supervisor_guard = supervisor_arc_clone.lock().await;
        supervisor_guard.subscribe(self, channel).await
    }

    /// Get a reference to a transport being used.
    pub fn transport(&self) -> &TTransport {
        &self.transport
    }

    /// Get a reference to a runtime being used.
    pub fn runtime(&self) -> &TRuntime {
        &self.runtime
    }
}
