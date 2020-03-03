use pubnub_hyper::core::data::{channel, request};
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::Builder;

mod common;

const SAMPLE_UUID: &'static str = "903145ee-7c15-4579-aa5d-38a900717512";

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
            let val = pubnub
                .call(request::SetState {
                    channels: vec![test_channel.clone()],
                    channel_groups: vec![],
                    uuid: SAMPLE_UUID.into(),
                    state: sample_state.clone(),
                })
                .await;
            assert_eq!(val.unwrap(), ());
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
