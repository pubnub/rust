//! TODO: Add documentation
use crate::PubNubClient;
use derive_builder::Builder;
use pubnub_core::{PubNubError, TransportMethod, TransportRequest};
use std::collections::HashMap;

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
    #[builder(default = "true")]
    replicate: bool,
    /// TODO: Add documentation
    ttl: u32,
    /// TODO: Add documentation
    #[builder(default = "false")]
    use_post: bool,
    /// TODO: Add documentation
    #[builder(default = "HashMap::new()")]
    meta: HashMap<String, String>,
}

impl<'pub_nub> PublishMessageViaChannelBuilder<'pub_nub> {
    /// TODO: Add documentation
    pub async fn execute(&self) -> Result<PublishResult, PubNubError> {
        if self.use_post == true {
            TransportRequest {
                path: format!("publish/{pubKey}/{subKey}/0/{channel}/0"),
                method: TransportMethod::Post,
                //body: self.message.unwrap(), TODO
                ..Default::default()
            }
        } else {
            TransportRequest {
                path: format!("publish/{}/{}/0/{}/0/{}", sub_key, pub_key, self.cha),
                method: TransportMethod::Get,
                ..Default::default()
            }
        }
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
