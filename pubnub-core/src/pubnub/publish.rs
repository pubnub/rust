use super::PubNub;
use crate::data::channel;
use crate::data::object::Object;
use crate::data::request;
use crate::data::timetoken::Timetoken;
use crate::runtime::Runtime;
use crate::transport::Transport;

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
    /// use pubnub_core::{data::channel, json::object, Builder};
    ///
    /// # async {
    /// let pubnub = Builder::with_components(transport, runtime).build();
    ///
    /// let channel_name: channel::Name = "my-channel".parse().unwrap();
    /// let timetoken = pubnub
    ///     .publish(
    ///         channel_name,
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
        channel: channel::Name,
        message: Object,
    ) -> Result<Timetoken, <TTransport as Transport>::Error> {
        let request = request::Publish {
            channel,
            meta: None,
            payload: message,
        };
        self.transport.call(request).await
    }

    /// Publish a message over the PubNub network with an extra metadata payload.
    ///
    /// # Example
    ///
    /// ```
    /// # use pubnub_core::mock::{transport::MockTransport, runtime::MockRuntime};
    /// # let transport = MockTransport::new();
    /// # let runtime = MockRuntime::new();
    /// use pubnub_core::{data::channel, json::object, Builder};
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
    /// let channel_name: channel::Name = "my-channel".parse().unwrap();
    /// let timetoken = pubnub
    ///     .publish_with_metadata(channel_name, message, metadata)
    ///     .await?;
    ///
    /// println!("Timetoken: {}", timetoken);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub async fn publish_with_metadata(
        &self,
        channel: channel::Name,
        message: Object,
        metadata: Object,
    ) -> Result<Timetoken, <TTransport as Transport>::Error> {
        let request = request::Publish {
            channel,
            meta: Some(metadata),
            payload: message,
        };
        self.transport.call(request).await
    }
}
