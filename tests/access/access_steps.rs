use crate::common::{common_steps::PAMCurrentResourceType, PubNubWorld};
use cucumber::{given, then, when};
use pubnub::{
    core::PubNubError,
    dx::{access::permissions, parse_token},
};
use std::collections::HashMap;
use std::ops::Not;

/// Add permission to specified `resource` in `resource` or `pattern`
/// permissions list.
fn add_permission(
    world: &mut PubNubWorld,
    name: Option<&String>,
    resource: Option<&String>,
    category: &String,
    permission: &String,
) {
    // Update current resource type if required.
    if let Some(resource) = resource {
        world.pam_state.resource_type = resource.into();
    }
    // Identify place, where permission should be stored.
    let resources = if category == "resource" {
        &mut world.pam_state.resource_permissions
    } else {
        &mut world.pam_state.pattern_permissions
    };

    match &world.pam_state.resource_type {
        PAMCurrentResourceType::Channel => {
            if let Some(name) = name {
                resources.channels.push(permissions::channel(name));
            }

            if permission.is_empty() {
                return;
            }

            if let Some(perm) = resources.channels.pop() {
                match permission.as_str() {
                    "READ" => resources.channels.push(perm.read()),
                    "WRITE" => resources.channels.push(perm.write()),
                    "GET" => resources.channels.push(perm.get()),
                    "MANAGE" => resources.channels.push(perm.manage()),
                    "UPDATE" => resources.channels.push(perm.update()),
                    "JOIN" => resources.channels.push(perm.join()),
                    "DELETE" => resources.channels.push(perm.delete()),
                    &_ => {}
                }
            }
        }
        PAMCurrentResourceType::ChannelGroup => {
            if let Some(name) = name {
                resources.groups.push(permissions::channel_group(name));
            }

            if permission.is_empty() {
                return;
            }

            if let Some(perm) = resources.groups.pop() {
                match permission.as_str() {
                    "READ" => resources.groups.push(perm.read()),
                    "MANAGE" => resources.groups.push(perm.manage()),
                    &_ => {}
                }
            }
        }
        PAMCurrentResourceType::UserId => {
            if let Some(name) = name {
                resources.user_ids.push(permissions::user_id(name));
            }

            if permission.is_empty() {
                return;
            }

            if let Some(perm) = resources.user_ids.pop() {
                match permission.as_str() {
                    "GET" => resources.user_ids.push(perm.get()),
                    "UPDATE" => resources.user_ids.push(perm.update()),
                    "DELETE" => resources.user_ids.push(perm.delete()),
                    &_ => {}
                }
            }
        }
        _ => {}
    }
}

#[given("a token")]
fn given_token(world: &mut PubNubWorld) {
    given_token_string(world, "valid access token".to_string());
}

#[given(regex = r"^the token string '(.*)'$")]
fn given_token_string(world: &mut PubNubWorld, token: String) {
    world.pam_state.access_token = Some(token);
}

#[given("I have a known token containing an authorized UUID")]
#[given("I have a known token containing UUID resource permissions")]
#[given("I have a known token containing UUID pattern Permissions")]
#[given(regex = r"^a( valid|n expired) token with permissions to publish with channel .*$")]
#[given("The SDK is configured with an AuthKey representing an access Token")]
#[given("I have associated an access token with the SDK instance")]
fn given_token_with_authorized_uuid(world: &mut PubNubWorld) {
    given_token_string(world,
                       "qEF2AkF0GmEI03xDdHRsGDxDcmVzpURjaGFuoWljaGFubmVsLTEY70NncnChb2NoYW5uZWxfZ3JvdXAtMQVDdXNyoENzcGOgRHV1aWShZnV1aWQtMRhoQ3BhdKVEY2hhbqFtXmNoYW5uZWwtXFMqJBjvQ2dycKF0XjpjaGFubmVsX2dyb3VwLVxTKiQFQ3VzcqBDc3BjoER1dWlkoWpedXVpZC1cUyokGGhEbWV0YaBEdXVpZHR0ZXN0LWF1dGhvcml6ZWQtdXVpZENzaWdYIPpU-vCe9rkpYs87YUrFNWkyNq8CVvmKwEjVinnDrJJc".to_string());
}

#[given(regex = r#"^the authorized UUID "(.*)"$"#)]
fn given_authorized_uuid(world: &mut PubNubWorld, authorized_uuid: String) {
    world.pam_state.authorized_uuid = authorized_uuid.is_empty().not().then_some(authorized_uuid);
}

#[given(regex = r"^the TTL (\d+)$")]
fn given_token_ttl(world: &mut PubNubWorld, ttl: usize) {
    world.pam_state.ttl = Some(ttl);
}

#[given(regex = r"^the '(.*)' (CHANNEL|CHANNEL_GROUP|UUID) (resource|pattern) access permissions$")]
fn given_grant_token_resource(
    world: &mut PubNubWorld,
    name: String,
    resource: String,
    category: String,
) {
    add_permission(
        world,
        Some(&name),
        Some(&resource),
        &category,
        &String::new(),
    );
}

#[given(
    regex = r"^grant (resource|pattern) permission (READ|WRITE|MANAGE|DELETE|GET|UPDATE|JOIN)$"
)]
fn given_resource_permission(world: &mut PubNubWorld, category: String, permission: String) {
    add_permission(world, None, None, &category, &permission);
}

#[when(regex = r"^I (attempt to )?grant a token specifying those permissions")]
async fn grant_given_permissions(world: &mut PubNubWorld, _unused: String) {
    let resource_permissions = world.pam_state.resource_permissions.permissions();
    let pattern_permissions = world.pam_state.pattern_permissions.permissions();
    let client = world.get_pubnub(world.keyset.to_owned());
    let mut builder = client
        .grant_token(world.pam_state.ttl.unwrap())
        .resources(&resource_permissions)
        .patterns(&pattern_permissions);

    // Adding authorized user identifier if provided.
    if let Some(authorised_uuid) = &world.pam_state.authorized_uuid {
        builder = builder.authorized_user_id(authorised_uuid);
    }

    world.pam_state.grant_token_result = builder
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        })
        .map(|response| {
            world.pam_state.access_token = Some(response.token.clone());
            response
        });

    world.is_succeed = world.pam_state.grant_token_result.is_ok();
}

#[when("I revoke a token")]
async fn i_revoke_a_token(world: &mut PubNubWorld) {
    // Intentional `unwrap` to panic if for some reason step with token
    // specification not called.
    let token = world.pam_state.access_token.clone().unwrap();
    world.pam_state.revoke_token_result = world
        .get_pubnub(world.keyset.to_owned())
        .revoke_token(token)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.pam_state.revoke_token_result.is_ok();
}

#[then("I get confirmation that token has been revoked")]
fn i_receive_token_revoke_confirmation(world: &mut PubNubWorld) {
    assert!(world.is_succeed, "Expected successful response");
}

#[then(regex = r#"^the token contains the authorized UUID "(.*)"$"#)]
#[then(regex = r#"^the parsed token output contains the authorized UUID "(.*)"$"#)]
fn token_contains_authorization_uuid(world: &mut PubNubWorld, uuid: String) {
    let token_string = world.pam_state.access_token.clone().unwrap();
    let mut matched = false;
    if let Ok(parse_token::Token::V2(token)) = parse_token(token_string.as_str()) {
        assert!(token.authorized_user_id.is_some());
        assert_eq!(token.authorized_user_id.unwrap(), uuid);
        matched = true;
    }
    assert!(matched);
}

#[then("the token does not contain an authorized uuid")]
fn token_not_contains_authorization_uuid(world: &mut PubNubWorld) {
    let token_string = world.pam_state.access_token.clone().unwrap();
    let mut matched = false;
    if let Ok(parse_token::Token::V2(token)) = parse_token(token_string.as_str()) {
        assert!(token.authorized_user_id.is_none());
        matched = true;
    }
    assert!(matched);
}

#[then(regex = r"^the token contains the TTL (\d+)$")]
fn token_contains_ttl(world: &mut PubNubWorld, ttl: u32) {
    let token_string = world.pam_state.access_token.clone().unwrap();
    let mut matched = false;
    if let Ok(parse_token::Token::V2(token)) = parse_token(token_string.as_str()) {
        assert!(token.ttl.gt(&0));
        assert_eq!(token.ttl, ttl);
        matched = true;
    }
    assert!(matched);
}

#[then(
    regex = r"^the token has '(.*)' (CHANNEL|CHANNEL_GROUP|UUID) (resource|pattern) access permissions$"
)]
fn token_contains_resource(
    world: &mut PubNubWorld,
    name: String,
    resource: String,
    category: String,
) {
    let token_string = world.pam_state.access_token.clone().unwrap();
    let mut matched = false;

    if let Ok(parse_token::Token::V2(token)) = parse_token(token_string.as_str()) {
        let resources = if category == "resource" {
            token.resources
        } else {
            token.patterns
        };

        if resource == "CHANNEL" && resources.channels.contains_key(name.as_str()) {
            world.pam_state.resource_type = PAMCurrentResourceType::Channel;
            world.pam_state.resource_name = Some(name);
            matched = true;
        } else if resource == "CHANNEL_GROUP" && resources.groups.contains_key(name.as_str()) {
            world.pam_state.resource_type = PAMCurrentResourceType::ChannelGroup;
            world.pam_state.resource_name = Some(name);
            matched = true;
        } else if resource == "UUID" && resources.users.contains_key(name.as_str()) {
            world.pam_state.resource_type = PAMCurrentResourceType::UserId;
            world.pam_state.resource_name = Some(name);
            matched = true;
        }
    }
    assert!(matched);
}

#[given(
    regex = r"^token (resource|pattern) permission (READ|WRITE|MANAGE|DELETE|GET|UPDATE|JOIN)$"
)]
fn token_resource_has_permission(world: &mut PubNubWorld, category: String, permission: String) {
    let resource_name = world.pam_state.resource_name.clone().unwrap();
    let token_string = world.pam_state.access_token.clone().unwrap();
    let mut matched = false;

    if let Ok(parse_token::Token::V2(token)) = parse_token(token_string.as_str()) {
        let resources = if category == "resource" {
            token.resources
        } else {
            token.patterns
        };
        let resource_permission = match world.pam_state.resource_type {
            PAMCurrentResourceType::Channel => resources.channels,
            PAMCurrentResourceType::ChannelGroup => resources.groups,
            PAMCurrentResourceType::UserId => resources.users,
            PAMCurrentResourceType::None => HashMap::new(),
        };
        let token_permission = resource_permission.get(resource_name.as_str()).unwrap();
        matched = true;

        match permission.as_str() {
            "READ" => assert!(token_permission.read),
            "WRITE" => assert!(token_permission.write),
            "GET" => assert!(token_permission.get),
            "MANAGE" => assert!(token_permission.manage),
            "UPDATE" => assert!(token_permission.update),
            "JOIN" => assert!(token_permission.join),
            "DELETE" => assert!(token_permission.delete),
            &_ => matched = false,
        }
    }
    assert!(matched);
}

#[given(regex = r"^deny resource permission (READ|WRITE|MANAGE|DELETE|GET|UPDATE|JOIN)$")]
fn deny_token_permission(_world: &mut PubNubWorld, _permission: String) {
    // Don't add any permissions.
}

#[when("I parse the token")]
fn i_parse_token(_world: &mut PubNubWorld) {
    // Do nothing, actual parsing happens in step.
}
