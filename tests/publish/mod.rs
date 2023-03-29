use cucumber::{then, when};
use std::collections::HashMap;

use crate::common::PubNubWorld;

#[when(expr = "I publish '{word}' string as message to '{word}' channel")]
async fn i_publish_string_as_message_to_channel(
    world: &mut PubNubWorld,
    message: String,
    channel: String,
) {
    world.last_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message)
        .channel(channel)
        .execute()
        .await;
}
#[then("I receive successful response")]
fn i_receive_successful_response(world: &mut PubNubWorld) {
    assert!(world.last_result.is_ok())
}

#[when(expr = "I publish {string} dictionary as message to '{word}' channel with compression")]
fn i_publish_dictionary_as_message_to_channel_with_compression(
    _world: &mut PubNubWorld,
    _dictionary_json: String,
    _channel: String,
) {
}

#[when(regex = r"^I publish '(.*)' dictionary as message to '(.*)' channel as POST body$")]
async fn i_publish_dictionary_as_message_to_channel_as_post_body(
    world: &mut PubNubWorld,
    dictionary_json: String,
    channel: String,
) {
    let message_hash_map: HashMap<String, String> =
        serde_json::from_str(dictionary_json.as_str()).unwrap();
    world.last_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_hash_map)
        .channel(channel)
        .use_post(true)
        .execute()
        .await;
}

#[when(regex = r"^I publish '(.*)' dictionary as message to '(.*)' channel$")]
async fn i_publish_dictionary_as_message_to_channel(
    world: &mut PubNubWorld,
    dictionary_json: String,
    channel: String,
) {
    let message_hash_map: HashMap<String, String> =
        serde_json::from_str(dictionary_json.as_str()).unwrap();
    world.last_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_hash_map)
        .channel(channel)
        .execute()
        .await;
}

#[when(regex = r"^I publish '(.*)' array as message to '(.*)' channel$")]
async fn i_publish_array_as_message_to_channel(
    world: &mut PubNubWorld,
    array_str: String,
    channel: String,
) {
    let message_array: [String; 2] = serde_json::from_str(array_str.as_str()).unwrap();
    world.last_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(message_array)
        .channel(channel)
        .execute()
        .await;
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
            world.last_result = world
                .get_pubnub(world.keyset.to_owned())
                .publish_message(message)
                .channel(channel)
                .meta(meta_map)
                .execute()
                .await;
        }
        "store" => {
            let store: bool = param_value != "0";
            world.last_result = world
                .get_pubnub(world.keyset.to_owned())
                .publish_message(message)
                .channel(channel)
                .store(store)
                .execute()
                .await;
        }
        _ => { /* do nothing */ }
    }
}

#[when(expr = "I publish '{int}' number as message to '{word}' channel")]
async fn i_publish_number_as_message_to_channel(
    world: &mut PubNubWorld,
    number: i32,
    channel: String,
) {
    world.last_result = world
        .get_pubnub(world.keyset.to_owned())
        .publish_message(number)
        .channel(channel)
        .execute()
        .await;
}
