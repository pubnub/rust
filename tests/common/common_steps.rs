use cucumber::gherkin::Scenario;
use cucumber::{given, then, World};
use pubnub::core::RequestRetryPolicy;
use pubnub::dx::subscribe::subscription::Subscription;
use pubnub::{
    core::PubNubError,
    dx::{
        access::{permissions, GrantTokenResult, RevokeTokenResult},
        publish::PublishResult,
    },
    transport::TransportReqwest,
    Keyset, PubNubClient, PubNubClientBuilder,
};
use std::fmt::Debug;

/// Type of resource for which permissions currently configured.
#[derive(Default, Debug)]
pub enum PAMCurrentResourceType {
    Channel,
    ChannelGroup,
    UserId,
    #[default]
    None,
}

impl From<&String> for PAMCurrentResourceType {
    fn from(value: &String) -> Self {
        if value == "CHANNEL" {
            Self::Channel
        } else if value == "CHANNEL_GROUP" {
            Self::ChannelGroup
        } else if value == "UUID" {
            Self::UserId
        } else {
            Self::None
        }
    }
}

/// PAM token configuration requires repetitive objects, so it better to be
/// stored as structure.
#[derive(Default, Debug)]
pub struct PAMPermissions {
    pub channels: Vec<permissions::ChannelPermission>,
    pub groups: Vec<permissions::ChannelGroupPermission>,
    pub user_ids: Vec<permissions::UserIdPermission>,
}

impl PAMPermissions {
    pub fn permissions(&self) -> Vec<Box<dyn permissions::Permission>> {
        let mut vec: Vec<Box<dyn permissions::Permission>> = vec![];
        self.channels.iter().for_each(|el| {
            vec.push(Box::new(permissions::ChannelPermission {
                name: el.name.clone(),
                bits: el.bits,
            }))
        });
        self.groups.iter().for_each(|el| {
            vec.push(Box::new(permissions::ChannelGroupPermission {
                name: el.name.clone(),
                bits: el.bits,
            }))
        });
        self.user_ids.iter().for_each(|el| {
            vec.push(Box::new(permissions::UserIdPermission {
                id: el.id.clone(),
                bits: el.bits,
            }))
        });

        vec
    }
}

/// World state for PAM.
#[derive(Debug)]
pub struct PAMState {
    pub revoke_token_result: Result<RevokeTokenResult, PubNubError>,
    pub grant_token_result: Result<GrantTokenResult, PubNubError>,
    pub resource_type: PAMCurrentResourceType,
    pub resource_permissions: PAMPermissions,
    pub pattern_permissions: PAMPermissions,
    pub authorized_uuid: Option<String>,
    pub resource_name: Option<String>,
    pub access_token: Option<String>,
    pub ttl: Option<usize>,
}

impl Default for PAMState {
    fn default() -> Self {
        Self {
            revoke_token_result: Err(PubNubError::Transport {
                details: "This is default value".into(),
                response: None,
            }),
            grant_token_result: Err(PubNubError::Transport {
                details: "This is default value".into(),
                response: None,
            }),
            resource_type: PAMCurrentResourceType::default(),
            resource_permissions: PAMPermissions::default(),
            pattern_permissions: PAMPermissions::default(),
            authorized_uuid: None,
            resource_name: None,
            access_token: None,
            ttl: None,
        }
    }
}

#[derive(Debug, World)]
pub struct PubNubWorld {
    pub scenario: Option<Scenario>,
    pub keyset: pubnub::Keyset<String>,
    pub publish_result: Result<PublishResult, PubNubError>,
    pub subscription: Result<Subscription, PubNubError>,
    pub retry_policy: Option<RequestRetryPolicy>,
    pub pam_state: PAMState,
    pub api_error: Option<PubNubError>,
    pub is_succeed: bool,
}

impl Default for PubNubWorld {
    fn default() -> Self {
        PubNubWorld {
            scenario: None,
            keyset: Keyset::<String> {
                subscribe_key: "demo".to_owned(),
                publish_key: Some("demo".to_string()),
                secret_key: Some("demo".to_string()),
            },
            publish_result: Err(PubNubError::Transport {
                details: "This is default value".into(),
                response: None,
            }),
            subscription: Err(PubNubError::Transport {
                details: "This is default value".into(),
                response: None,
            }),
            is_succeed: false,
            pam_state: PAMState::default(),
            api_error: None,
            retry_policy: None,
        }
    }
}

impl PubNubWorld {
    pub fn get_pubnub(&self, keyset: Keyset<String>) -> PubNubClient {
        let transport = {
            let mut transport = TransportReqwest::default();
            transport.hostname = "http://localhost:8090".into();
            transport
        };

        let mut builder = PubNubClientBuilder::with_transport(transport)
            .with_keyset(keyset)
            .with_user_id("test");

        if let Some(retry_policy) = &self.retry_policy {
            builder = builder.with_retry_policy(retry_policy.clone());
        }

        builder.build().unwrap()
    }
}

#[given("the demo keyset")]
#[given("the demo keyset with event engine enabled")]
fn set_keyset(world: &mut PubNubWorld) {
    world.keyset = Keyset {
        subscribe_key: "demo".into(),
        publish_key: Some("demo".into()),
        secret_key: None,
    }
}

#[given(regex = r"^a (.*) reconnection policy with ([0-9]+) retries")]
fn set_with_retries(world: &mut PubNubWorld, retry_type: String, max_retry: u8) {
    if retry_type.eq("linear") {
        world.retry_policy = Some(RequestRetryPolicy::Linear {
            max_retry,
            delay: 0,
        })
    }
}

#[given(regex = r"^I have a keyset with access manager enabled(.*)?")]
fn i_have_keyset_with_access_manager_enabled(world: &mut PubNubWorld, info: String) {
    world.keyset = Keyset {
        subscribe_key: "demo".into(),
        publish_key: Some("demo".into()),
        secret_key: if info.contains("without") {
            None
        } else {
            Some("demo".into())
        },
    }
}

#[then("I receive an error response")]
fn i_receive_error_response(world: &mut PubNubWorld) {
    assert!(!world.is_succeed)
}

#[then("the result is successful")]
fn i_receive_success_response(world: &mut PubNubWorld) {
    assert!(world.is_succeed)
}

#[then("an error is returned")]
fn an_error_is_returned(world: &mut PubNubWorld) {
    assert!(world.api_error.is_some(), "Expected to receive API error");
}

#[then("an auth error is returned")]
fn an_auth_error_is_returned(world: &mut PubNubWorld) {
    assert!(world.api_error.is_some(), "Expected to receive API error");
    if let Some(PubNubError::API {
        status, service, ..
    }) = &world.api_error
    {
        assert_eq!(status, &403);
        if let Some(service) = service {
            assert_eq!(service, "Access Manager");
        }
    } else {
        assert!(world.api_error.is_some(), "Unexpected to receive API error");
    }
}

#[then(regex = r"^the error status code is (\d+)$")]
#[given(regex = r"^the error status code is (\d+)$")]
fn has_specific_error_code(world: &mut PubNubWorld, expected_status_code: u16) {
    if let Some(PubNubError::API { status, .. }) = world.api_error.clone() {
        assert_eq!(status, expected_status_code);
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error message is '(.*)'$")]
#[given(regex = r"^the error message is '(.*)'$")]
#[given(regex = r"^the auth error message is '(.*)'$")]
fn contains_specific_error_message(world: &mut PubNubWorld, expected_message: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(
            message.contains(expected_message.as_str()),
            "Expected '{expected_message}', but got '{message}'"
        );
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error service is '(.*)'$")]
#[given(regex = r"^the error service is '(.*)'$")]
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

#[then(regex = r"^the error source is '(.*)'$")]
#[given(regex = r"^the error source is '(.*)'$")]
fn has_specific_source(_world: &mut PubNubWorld) {
    // noop
}

#[then("the error detail message is not empty")]
#[given("the error detail message is not empty")]
fn message_not_empty(world: &mut PubNubWorld) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert_ne!(message.len(), 0, "Error message shouldn't be empty.");
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error detail message is '(.*)'$")]
#[given(regex = r"^the error detail message is '(.*)'$")]
fn has_error_information(world: &mut PubNubWorld, expected_information: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(expected_information.as_str()));
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error detail location is '(.*)'$")]
#[given(regex = r"^the error detail location is '(.*)'$")]
fn has_error_location_information(world: &mut PubNubWorld, expected_location: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(format!("name: '{expected_location}'").as_str()));
    } else {
        panic!("API error is missing");
    }
}

#[then(regex = r"^the error detail location type is '(.*)'$")]
#[given(regex = r"^the error detail location type is '(.*)'$")]
fn has_error_location_type_information(world: &mut PubNubWorld, expected_location_type: String) {
    if let PubNubError::API { message, .. } = world.api_error.clone().unwrap() {
        assert!(message.contains(format!("location: '{expected_location_type}'").as_str()));
    } else {
        panic!("API error is missing");
    }
}
