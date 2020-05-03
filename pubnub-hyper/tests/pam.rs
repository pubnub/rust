use pubnub_hyper::core::data::{pam, request};
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::Builder;
use std::collections::HashMap;

mod common;

#[test]
fn grant() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .secret_key("wMfbo9G0xVUG8yfTfYw5qIdfJkTd7A")
            .build()
            .unwrap();

        let pubnub = Builder::with_components(transport, TokioGlobal).build();

        {
            pubnub
                .call(request::Grant {
                    ttl: 10,
                    permissions: pam::Permissions {
                        resources: pam::Resources {
                            channels: {
                                let mut map = HashMap::new();
                                map.insert("channel_a".into(), pam::BitMask(4));
                                map.insert("channel_b".into(), pam::BitMask(0));
                                map
                            },
                            groups: {
                                let mut map = HashMap::new();
                                map.insert("groups_a".into(), pam::BitMask(4));
                                map.insert("groups_b".into(), pam::BitMask(0));
                                map
                            },
                            users: {
                                let mut map = HashMap::new();
                                map.insert("users_a".into(), pam::BitMask(4));
                                map.insert("users_b".into(), pam::BitMask(0));
                                map
                            },
                            spaces: {
                                let mut map = HashMap::new();
                                map.insert("spaces_a".into(), pam::BitMask(4));
                                map.insert("spaces_b".into(), pam::BitMask(0));
                                map
                            },
                        },
                        patterns: pam::Patterns {
                            channels: {
                                let mut map = HashMap::new();
                                map.insert("channel_c".into(), pam::BitMask(4));
                                map.insert("channel_d".into(), pam::BitMask(0));
                                map
                            },
                            groups: {
                                let mut map = HashMap::new();
                                map.insert("groups_c".into(), pam::BitMask(4));
                                map.insert("groups_d".into(), pam::BitMask(0));
                                map
                            },
                            users: {
                                let mut map = HashMap::new();
                                map.insert("users_c".into(), pam::BitMask(4));
                                map.insert("users_d".into(), pam::BitMask(0));
                                map
                            },
                            spaces: {
                                let mut map = HashMap::new();
                                map.insert("spaces_c".into(), pam::BitMask(4));
                                map.insert("spaces_d".into(), pam::BitMask(0));
                                map
                            },
                        },
                        meta: json::object! {
                            "user_id" => "qwerty",
                        },
                    },
                })
                .await
                .unwrap();
        }
    });
}
