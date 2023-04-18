use cucumber::{given, then, when};
use futures::future::err;
use pubnub::core::PubNubError;
use pubnub::Keyset;

use crate::common::PubNubWorld;

#[given("a token")]
fn given_token(world: &mut PubNubWorld) {
    world.access_token = Some("valid access token".into());
}

#[when("I revoke a token")]
async fn i_revoke_a_token(world: &mut PubNubWorld) {
    // Intentional `unwrap` to panic if for some reason step with token
    // specification not called.
    let token = world.access_token.clone().unwrap();
    world.revoke_token_result = world
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
    world.is_succeed = world.revoke_token_result.is_ok();
}

#[then("I get confirmation that token has been revoked")]
fn i_receive_token_revoke_confirmation(world: &mut PubNubWorld) {
    assert!(world.is_succeed, "Expected successful response");
}
