use crate::common::{common_steps::PAMCurrentResourceType, PubNubWorld};
use cucumber::{given, then, when};
use pubnub::{core::PubNubError, dx::access::permissions};
use std::ops::Not;

/// Retrieve permission
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
                if permission == "READ" {
                    resources.channels.push(perm.read());
                } else if permission == "WRITE" {
                    resources.channels.push(perm.write());
                } else if permission == "GET" {
                    resources.channels.push(perm.get());
                } else if permission == "MANAGE" {
                    resources.channels.push(perm.manage());
                } else if permission == "UPDATE" {
                    resources.channels.push(perm.update());
                } else if permission == "JOIN" {
                    resources.channels.push(perm.join());
                } else if permission == "DELETE" {
                    resources.channels.push(perm.delete());
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
                if permission == "READ" {
                    resources.groups.push(perm.read());
                } else if permission == "MANAGE" {
                    resources.groups.push(perm.manage());
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
                if permission == "GET" {
                    resources.user_ids.push(perm.get());
                } else if permission == "UPDATE" {
                    resources.user_ids.push(perm.update());
                } else if permission == "DELETE" {
                    resources.user_ids.push(perm.delete());
                }
            }
        }
        _ => {}
    }
}

#[given("a token")]
fn given_token(world: &mut PubNubWorld) {
    world.pam_state.access_token = Some("valid access token".into());
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

#[when("I grant a token specifying those permissions")]
async fn grant_given_permissions(world: &mut PubNubWorld) {
    let resource_permissions = world.pam_state.resource_permissions.permissions();
    let pattern_permissions = world.pam_state.pattern_permissions.permissions();
    let result = world
        .get_pubnub(world.keyset.to_owned())
        .grant_token(world.pam_state.ttl.unwrap())
        .resources(&resource_permissions)
        .patterns(&pattern_permissions)
        .authorized_user_id(world.pam_state.authorized_uuid.clone().unwrap())
        .execute()
        .await;

    world.is_succeed = world.publish_result.is_ok();
    world.pam_state.access_token = result.is_ok().then_some(result.unwrap().token);
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

#[then(regex = r#"the token contains the authorized UUID "(.*)""#)]
fn token_contains_authorization_uuid(world: &mut PubNubWorld, uuid: String) {
    // world.pam_state.revoke_token_result = world.get_pubnub(world.keyset.to_owned())
}
