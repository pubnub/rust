use log::info;
use pubnub_hyper::core::data::{channel, request};
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::Builder;
use std::fmt::Write;

mod common;

/// Delay execution for the specified amount of milliseconds.
async fn sleep(ms: u64) {
    info!(target: "pubnub", "Sleeping for {} ms", ms);
    tokio::time::delay_for(std::time::Duration::from_millis(ms)).await
}

/// Generate a string of random numbers.
fn random_hex_string() -> String {
    use getrandom::getrandom;
    let mut buf = [0u8; 4];
    getrandom(&mut buf).expect("failed to getrandom");
    let mut s = String::new();
    for b in buf.iter() {
        write!(s, "{:02X}", b).unwrap();
    }
    s
}

#[test]
fn get_history() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();

        let test_channel: channel::Name = format!("my-channel-{}", random_hex_string())
            .parse()
            .unwrap();
        let test_payload = json::object! {
            my_payload: "my_value",
        };
        let test_metadata = json::object! {
            my_metadata: "my_value",
        };

        {
            pubnub
                .call(request::Publish {
                    channel: test_channel.clone(),
                    payload: test_payload.clone(),
                    meta: Some(test_metadata.clone()),
                })
                .await
                .unwrap();
        }

        // Give the network some time to react.
        sleep(1000).await;

        {
            let channels = pubnub
                .call(request::GetHistory {
                    channels: vec![test_channel.clone()],
                    max: None,
                    reverse: None,
                    start: None,
                    end: None,
                    include_metadata: None,
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 1);

            let test_channel_messages = &channels[&test_channel];
            assert_eq!(test_channel_messages.len(), 1);

            let item = &test_channel_messages[0];
            assert_eq!(&item.message, &test_payload);
            assert_ne!(item.timetoken, 0);
            assert_eq!(&item.metadata, &json::Null);
        }

        {
            let channels = pubnub
                .call(request::GetHistory {
                    channels: vec![test_channel.clone()],
                    max: None,
                    reverse: None,
                    start: None,
                    end: None,
                    include_metadata: Some(true),
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 1);

            let test_channel_messages = &channels[&test_channel];
            assert_eq!(test_channel_messages.len(), 1);

            let item = &test_channel_messages[0];
            assert_eq!(&item.message, &test_payload);
            assert_ne!(item.timetoken, 0);
            assert_eq!(&item.metadata, &test_metadata);
        }
    });
}

#[test]
fn delete_history() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();

        let test_channel: channel::Name = format!("my-channel-{}", random_hex_string())
            .parse()
            .unwrap();
        let test_payload = json::object! {
            my_payload: "my_value",
        };

        {
            pubnub
                .call(request::Publish {
                    channel: test_channel.clone(),
                    payload: test_payload.clone(),
                    meta: None,
                })
                .await
                .unwrap();
        }

        // Give the network some time to react.
        sleep(1000).await;

        {
            let channels = pubnub
                .call(request::GetHistory {
                    channels: vec![test_channel.clone()],
                    max: None,
                    reverse: None,
                    start: None,
                    end: None,
                    include_metadata: None,
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 1);

            let test_channel_messages = &channels[&test_channel];
            assert_eq!(test_channel_messages.len(), 1);
        }

        {
            pubnub
                .call(request::DeleteHistory {
                    channels: vec![test_channel.clone()],
                    start: None,
                    end: None,
                })
                .await
                .unwrap();
        }

        // Give the network some time to react.
        sleep(1000).await;

        {
            let channels = pubnub
                .call(request::GetHistory {
                    channels: vec![test_channel.clone()],
                    max: None,
                    reverse: None,
                    start: None,
                    end: None,
                    include_metadata: None,
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 0);
        }
    });
}

#[test]
fn message_counts() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();

        let test_channel: channel::Name = format!("my-channel-{}", random_hex_string())
            .parse()
            .unwrap();
        let test_payload = json::object! {
            my_payload: "my_value",
        };
        let test_timetoken = 3600 * 10_000_000;

        {
            pubnub
                .call(request::Publish {
                    channel: test_channel.clone(),
                    payload: test_payload.clone(),
                    meta: None,
                })
                .await
                .unwrap();
        }

        // Give the network some time to react.
        sleep(5000).await;

        {
            let channels = pubnub
                .call(request::MessageCountsWithTimetoken {
                    channels: vec![test_channel.clone()],
                    timetoken: test_timetoken,
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 1);
            assert_eq!(&channels[&test_channel], &1);
        }

        {
            let channels = pubnub
                .call(request::MessageCountsWithChannelTimetokens {
                    channels: vec![(test_channel.clone(), test_timetoken)]
                        .into_iter()
                        .collect(),
                })
                .await
                .unwrap();
            assert_eq!(channels.len(), 1);
            assert_eq!(&channels[&test_channel], &1);
        }
    });
}
