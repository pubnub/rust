use cucumber::{given, then, World};
use pubnub::{
    core::PubNubError,
    dx::{
        access::{GrantTokenResult, RevokeTokenResult},
        publish::PublishResult,
    },
    transport::{middleware::PubNubMiddleware, TransportReqwest},
    Keyset, PubNubClient, PubNubClientBuilder,
};
use std::fmt::format;

#[derive(Debug, World)]
pub struct PubNubWorld {
    pub keyset: pubnub::Keyset<String>,
    pub publish_result: Result<PublishResult, PubNubError>,
    pub grant_token_result: Result<GrantTokenResult, PubNubError>,
    pub revoke_token_result: Result<RevokeTokenResult, PubNubError>,
    pub access_token: Option<String>,
    pub api_error: Option<PubNubError>,
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
            publish_result: Err(PubNubError::Transport("This is default value".into())),
            grant_token_result: Err(PubNubError::Transport("This is default value".into())),
            revoke_token_result: Err(PubNubError::Transport("This is default value".into())),
            access_token: None,
            api_error: None,
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
            transport.hostname = "http://localhost:8090".into();
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

#[given("I have a keyset with access manager enabled")]
fn i_have_keyset_with_access_manager_enabled(world: &mut PubNubWorld) {
    world.keyset = Keyset {
        subscribe_key: "demo".into(),
        publish_key: Some("demo".into()),
        secret_key: Some("demo".into()),
    }
}

#[then("an error is returned")]
fn an_error_is_returned(world: &mut PubNubWorld) {
    assert!(world.api_error.is_some(), "Expected to receive API error");
}

#[then(regex = r"^the error status code is (\d+)$")]
fn has_specific_error_code(world: &mut PubNubWorld, expected_status_code: u16) {
    if let PubNubError::API { status, .. } = world.api_error.clone().unwrap() {
        assert_eq!(status, expected_status_code);
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error message is '(.*)'$")]
fn contains_specific_error_message(world: &mut PubNubWorld, expected_message: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(expected_message.as_str()));
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error service is '(.*)''$")]
fn has_specific_service(world: &mut PubNubWorld, expected_service: String) {
    if let PubNubError::API { service, .. } = world.api_error.clone().unwrap() {
        if let Some(service) = service {
            assert!(service.contains(expected_service.as_str()));
        } else {
            panic!("Service information is missing");
        }
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error source is '(.*)''$")]
fn has_specific_source(world: &mut PubNubWorld) {
    // noop
}

#[then("the error detail message is not empty")]
fn message_not_empty(world: &mut PubNubWorld) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert_ne!(message.len(), 0, "Error message shouldn't be empty.");
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error detail location is '(.*)'$")]
fn has_error_location_information(world: &mut PubNubWorld, expected_location: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(format!("name: '{expected_location}'").as_str()));
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error detail location type is '(.*)'$")]
fn has_error_location_type_information(world: &mut PubNubWorld, expected_location_type: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(format!("location: '{expected_location_type}'").as_str()));
    } else {
        panic!("API error is missing");
    }
}
