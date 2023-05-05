#![no_std]
extern crate alloc;

use alloc::string::String;
use pubnub::{core::PubNubError, Keyset, PubNubClientBuilder};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    content: String,
    author: String,
}

// As `no_std` does not support `Error` trait, we use `PubNubError` instead.
// In your program, you should handle the error properly for your use case.
fn main() -> Result<(), PubNubError> {
    // As `no_std` does not support `env::var`, you need to set the keys manually.
    let publish_key = "SDK_PUB_KEY";
    let subscribe_key = "SDK_SUB_KEY";

    let client = PubNubClientBuilder::with_reqwest_blocking_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .build()?;

    // `execute_blocking` returns result with the outcome of the operation.
    // As `no_std` does not support `println`, we omit the result in this example.
    // See more details in `publish_blocking.rs`.

    // publish simple string
    client
        .publish_message("hello world!")
        .channel("my_channel")
        .execute_blocking()?;

    // publish a struct
    client
        .publish_message(Message {
            content: "hello world!".into(),
            author: "me".into(),
        })
        .channel("my_channel")
        .execute_blocking()?;

    // publish with whole config options
    client
        .publish_message("hello with params!")
        .channel("my_channel")
        .store(true)
        .meta([("meta1".into(), "meta2".into())].into())
        .replicate(true)
        .use_post(true)
        .ttl(10)
        .space_id("my_space")
        .r#type("my_type")
        .execute_blocking()?;

    Ok(())
}
