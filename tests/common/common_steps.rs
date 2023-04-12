use cucumber::{given, World};
use pubnub::{
    core::PubNubError,
    dx::publish::PublishResult,
    transport::{middleware::PubNubMiddleware, TransportReqwest},
    Keyset, PubNubClient, PubNubClientBuilder,
};

#[derive(Debug, World)]
pub struct PubNubWorld {
    pub keyset: pubnub::Keyset<String>,
    pub publish_result: Result<PublishResult, PubNubError>,
    pub is_succeed: bool,
}

impl Default for PubNubWorld {
    fn default() -> Self {
        PubNubWorld {
            keyset: Keyset::<String> {
                subscribe_key: "demo".to_owned(),
                publish_key: Some("demo".to_string()),
                secret_key: Some("demo".to_string()),
            },
            publish_result: Err(PubNubError::TransportError("This is default value")),
            is_succeed: false,
        }
    }
}

impl PubNubWorld {
    pub fn get_pubnub(
        &self,
        keyset: Keyset<String>,
    ) -> PubNubClient<PubNubMiddleware<TransportReqwest>> {
        let transport = {
            let mut transport = TransportReqwest::default();
            transport.hostname = "http://localhost:8090/";
            transport
        };
        PubNubClientBuilder::with_reqwest_transport()
            .with_transport(transport)
            .with_keyset(keyset)
            .with_user_id("test")
            .build()
            .unwrap()
    }
}

#[given("the demo keyset")]
fn set_keyset(world: &mut PubNubWorld) {
    world.keyset.publish_key = Some("demo".to_string());
    world.keyset.subscribe_key = "demo".to_string();
}
