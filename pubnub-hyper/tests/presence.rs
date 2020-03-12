use log::info;
use pubnub_hyper::core::data::{
    channel, presence, pubsub, request, response, timetoken::Timetoken,
};
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::Builder;
use std::convert::TryInto;
use std::fmt::Write;

mod common;

const SAMPLE_UUID: &str = "903145ee-7c15-4579-aa5d-38a900717512";

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
fn get_set_state() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();

        let test_channel: channel::Name = "my-channel".parse().unwrap();
        let sample_state = json::object! {
            "my_sample_state" => 123,
        };

        {
            pubnub
                .call(request::SetState {
                    channels: vec![test_channel.clone()],
                    channel_groups: vec![],
                    uuid: SAMPLE_UUID.into(),
                    state: sample_state.clone(),
                })
                .await
                .unwrap();
        }

        {
            let val = pubnub
                .call(request::GetState {
                    channels: vec![test_channel.clone()],
                    channel_groups: vec![],
                    uuid: SAMPLE_UUID.into(),
                })
                .await;
            assert_eq!(val.unwrap(), sample_state);
        }
    });
}

#[test]
fn here_now_single_channel() {
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
            .try_into()
            .unwrap();

        {
            let val = pubnub
                .call(request::Subscribe {
                    to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                    timetoken: Timetoken::default(),
                })
                .await;
            assert!(val.is_ok());
        }

        // Wait for PunNub network to react.
        sleep(10000).await;

        {
            let val = pubnub
                .call(request::HereNow::<presence::respond_with::OccupancyOnly> {
                    channels: vec![test_channel.clone()],
                    channel_groups: vec![],
                    respond_with: std::marker::PhantomData,
                })
                .await
                .unwrap();
            assert_eq!(
                val,
                response::HereNow::<presence::respond_with::OccupancyOnly> { occupancy: 1 }
            );
        }

        {
            let val = pubnub
                .call(
                    request::HereNow::<presence::respond_with::OccupancyAndUUIDs> {
                        channels: vec![test_channel.clone()],
                        channel_groups: vec![],
                        respond_with: std::marker::PhantomData,
                    },
                )
                .await
                .unwrap();
            assert_eq!(
                val,
                response::HereNow::<presence::respond_with::OccupancyAndUUIDs> {
                    occupancy: 1,
                    occupants: vec![pubnub.transport().uuid().clone()]
                }
            );
        }

        {
            let val = pubnub
                .call(request::HereNow::<presence::respond_with::Full> {
                    channels: vec![test_channel.clone()],
                    channel_groups: vec![],
                    respond_with: std::marker::PhantomData,
                })
                .await
                .unwrap();
            assert_eq!(
                val,
                response::HereNow::<presence::respond_with::Full> {
                    occupancy: 1,
                    occupants: vec![presence::ChannelOccupantFullDetails {
                        uuid: pubnub.transport().uuid().clone(),
                        state: json::Null,
                    }]
                }
            );
        }
    });
}

#[test]
fn global_here_now() {
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
            .try_into()
            .unwrap();

        {
            let val = pubnub
                .call(request::Subscribe {
                    to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                    timetoken: Timetoken::default(),
                })
                .await;
            assert!(val.is_ok());
        }

        // Wait for PunNub network to react.
        sleep(60000).await;

        {
            let val = pubnub
                .call(
                    request::GlobalHereNow::<presence::respond_with::OccupancyOnly> {
                        respond_with: std::marker::PhantomData,
                    },
                )
                .await
                .unwrap();
            assert!(val.total_channels >= 1);
            assert!(val.total_occupancy >= 1);
            assert_eq!(
                val.channels[&test_channel],
                presence::ChannelInfo { occupancy: 1 }
            );
        }

        {
            let val = pubnub
                .call(
                    request::GlobalHereNow::<presence::respond_with::OccupancyAndUUIDs> {
                        respond_with: std::marker::PhantomData,
                    },
                )
                .await
                .unwrap();
            assert!(val.total_channels >= 1);
            assert!(val.total_occupancy >= 1);
            assert_eq!(
                val.channels[&test_channel],
                presence::ChannelInfoWithOccupants {
                    occupancy: 1,
                    occupants: vec![pubnub.transport().uuid().clone()],
                },
            );
        }

        {
            let val = pubnub
                .call(request::GlobalHereNow::<presence::respond_with::Full> {
                    respond_with: std::marker::PhantomData,
                })
                .await
                .unwrap();
            assert!(val.total_channels >= 1);
            assert!(val.total_occupancy >= 1);
            assert_eq!(
                val.channels[&test_channel],
                presence::ChannelInfoWithOccupants {
                    occupancy: 1,
                    occupants: vec![presence::ChannelOccupantFullDetails {
                        uuid: pubnub.transport().uuid().clone(),
                        state: json::Null,
                    }],
                },
            );
        }
    });
}

#[test]
fn where_now() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();
        let other_transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();
        let other_pubnub = Builder::with_components(other_transport, TokioGlobal).build();

        let test_channel: channel::Name = format!("my-channel-{}", random_hex_string())
            .try_into()
            .unwrap();

        {
            let val = other_pubnub
                .call(request::Subscribe {
                    to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                    timetoken: Timetoken::default(),
                })
                .await;
            assert!(val.is_ok());
        }

        // Wait for PunNub network to react.
        sleep(10000).await;

        {
            let val = pubnub
                .call(request::WhereNow {
                    uuid: other_pubnub.transport().uuid().clone(),
                })
                .await
                .unwrap();
            assert_eq!(val, vec![test_channel.clone()]);
        }
    });
}
