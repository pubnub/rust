//! TODO: Add documentation
use crate::PubNubClient;
use derive_builder::Builder;
use pubnub_core::PubNubError;

/// TODO: Add documentation
pub type MessageType = String;

/// TODO: Add documentation
pub struct PublishMessageBuilder<'pub_nub> {
    pub_nub_client: &'pub_nub PubNubClient,
    message: MessageType,
}

impl<'pub_nub> PublishMessageBuilder<'pub_nub> {
    /// TODO: Add documentation
    pub fn channel(self, channel: String) -> PublishMessageViaChannelBuilder<'pub_nub> {
        PublishMessageViaChannelBuilder {
            pub_nub_client: Some(self.pub_nub_client),
            ..Default::default()
        }
        .message(self.message)
        .channel(channel)
    }
}

/// TODO: Add documentation
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(skip))]
pub struct PublishMessageViaChannel<'pub_nub> {
    #[builder(setter(custom))]
    pub_nub_client: &'pub_nub PubNubClient,
    /// TODO: Add documentation
    message: MessageType,
    /// TODO: Add documentation
    channel: String,
    /// TODO: Add documentation
    store: bool,
    /// TODO: Add documentation
    replicate: bool,
    /// TODO: Add documentation
    ttl: u32,
}

impl<'pub_nub> PublishMessageViaChannelBuilder<'pub_nub> {
    /// TODO: Add documentation
    pub async fn execute(&self) -> Result<PublishResult, PubNubError> {
        Ok(PublishResult)
    }
}

struct PublishResult;

impl PubNubClient {
    /// TODO: Add documentation
    pub fn publish_message(&self, message: MessageType) -> PublishMessageBuilder {
        PublishMessageBuilder {
            message,
            pub_nub_client: self,
        }
    }
}

#[cfg(test)]
mod should {
    use crate::publish::PublishMessageViaChannelBuilder;
    use crate::PubNubClient;
    use pubnub::publish;

    fn test(instance: PubNubClient) {
        instance
            .publish_message("First message".into())
            .channel("Iguess".into())
            .replicate(true)
            .execute();

        instance
            .publish_message("I could send this message".into())
            .channel("chan".into())
            .store(true)
            .ttl(140)
            .execute();
    }
}
