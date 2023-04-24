use cucumber::{given, then, when};
use pubnub::core::PubNubError;
use std::collections::HashMap;

use crate::common::PubNubWorld;

#[given("the wrong publish key")]
fn set_wron_publish_key(world: &mut PubNubWorld) {
    world.keyset.publish_key = Some("wrong_key".to_string());
}

#[given("the wrong subscribe key")]
fn set_wrong_subscribe_key(world: &mut PubNubWorld) {
    world.keyset.subscribe_key = "wrong_key".to_string();
}

#[when(regex = "^I (attempt to )?publish a (.*) using that auth token with channel '(.*)'$")]
#[when(expr = "I publish '{word}' string as message to '{word}' channel")]
async fn i_publish_string_as_message_to_channel(
    world: &mut PubNubWorld,
    message: String,
    channel: String,
) {
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message)
        .channel(channel)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}

#[then("I receive successful response")]
fn i_receive_successful_response(world: &mut PubNubWorld) {
    assert!(world.is_succeed);
}

#[when(regex = r"^I publish '(.*)' dictionary as message to '(.*)' channel as POST body$")]
async fn i_publish_dictionary_as_message_to_channel_as_post_body(
    world: &mut PubNubWorld,
    dictionary_json: String,
    channel: String,
) {
    let message_hash_map: HashMap<String, String> =
        serde_json::from_str(dictionary_json.as_str()).unwrap();
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_hash_map)
        .channel(channel)
        .use_post(true)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}

#[when(regex = r"^I publish '(.*)' dictionary as message to '(.*)' channel$")]
async fn i_publish_dictionary_as_message_to_channel(
    world: &mut PubNubWorld,
    dictionary_json: String,
    channel: String,
) {
    let message_hash_map: HashMap<String, String> =
        serde_json::from_str(dictionary_json.as_str()).unwrap();
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_hash_map)
        .channel(channel)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}

#[when(regex = r"^I publish '(.*)' array as message to '(.*)' channel$")]
async fn i_publish_array_as_message_to_channel(
    world: &mut PubNubWorld,
    array_str: String,
    channel: String,
) {
    let message_array: [String; 2] = serde_json::from_str(array_str.as_str()).unwrap();
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_array)
        .channel(channel)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}

#[when(regex = r"^I publish '(.*)' string as message to '(.*)' channel with '(.*)' set to '(.*)'$")]
async fn i_publish_message_to_channel_with_meta(
    world: &mut PubNubWorld,
    message: String,
    channel: String,
    param: String,
    param_value: String,
) {
    match param.as_str() {
        "meta" => {
            let meta_map: HashMap<String, String> =
                serde_json::from_str(param_value.as_str()).unwrap();
            world.publish_result = world
                .get_pubnub(world.keyset.to_owned())
                .publish_message(message)
                .channel(channel)
                .meta(meta_map)
                .execute()
                .await
                .map_err(|err| {
                    if let PubNubError::API { .. } = err {
                        world.api_error = Some(err.clone());
                    }
                    err
                });
            world.is_succeed = world.publish_result.is_ok();
        }
        "store" => {
            let store: bool = param_value != "0";
            world.publish_result = world
                .get_pubnub(world.keyset.to_owned())
                .publish_message(message)
                .channel(channel)
                .store(store)
                .execute()
                .await
                .map_err(|err| {
                    if let PubNubError::API { .. } = err {
                        world.api_error = Some(err.clone());
                    }
                    err
                });
            world.is_succeed = world.publish_result.is_ok();
        }
        "ttl" => {
            let ttl: u32 = param_value.parse::<u32>().unwrap();
            world.publish_result = world
                .get_pubnub(world.keyset.to_owned())
                .publish_message(message)
                .channel(channel)
                .ttl(ttl)
                .execute()
                .await
                .map_err(|err| {
                    if let PubNubError::API { .. } = err {
                        world.api_error = Some(err.clone());
                    }
                    err
                });
            world.is_succeed = world.publish_result.is_ok();
        }
        _ => { /* do nothing */ }
    }
}

#[when(expr = "I publish too long message to '{word}' channel")]
async fn i_publish_too_long_message_to_channel(world: &mut PubNubWorld, channel: String) {
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message("this is too long mesage")
        .channel(channel)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}

#[when(expr = "I publish '{int}' number as message to '{word}' channel")]
async fn i_publish_number_as_message_to_channel(
    world: &mut PubNubWorld,
    number: i32,
    channel: String,
) {
    world.publish_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(number)
        .channel(channel)
        .execute()
        .await
        .map_err(|err| {
            if let PubNubError::API { .. } = err {
                world.api_error = Some(err.clone());
            }
            err
        });
    world.is_succeed = world.publish_result.is_ok();
}
