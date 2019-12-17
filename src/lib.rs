//! # Async PubNub Client SDK for Rust
//!
//! - Fully `async`/`await` ready.
//! - Uses Tokio and Hyper to provide an ultra-fast, incredibly reliable message transport over the
//!   PubNub edge network.
//! - Optimizes for minimal network sockets with an infinite number of logical streams.
//!
//! # Example
//!
//! ```
//! use futures_util::stream::StreamExt;
//! use pubnub::{json::object, PubNub};
//!
//! # async {
//! let mut pubnub = PubNub::new("demo", "demo");
//!
//! let message = object!{
//!     "username" => "JoeBob",
//!     "content" => "Hello, world!",
//! };
//!
//! let mut stream = pubnub.subscribe("my-channel").await;
//! let timetoken = pubnub.publish("my-channel", message.clone()).await?;
//!
//! let received = stream.next().await;
//! assert_eq!(received.unwrap().json, message);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # };
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

pub use crate::error::Error;
pub use crate::message::{Message, Timetoken, Type};
pub use crate::pubnub::{PubNub, PubNubBuilder};
pub use crate::subscribe::Subscription;
pub use json;

/// PubNub client with built-in defaults.
pub mod default {
    pub use crate::adapters::runtime::default::Runtime as DefaultRuntime;
    pub use crate::adapters::transport::default::Transport as DefaultTransport;

    #[deny(clippy::module_name_repetitions)]
    pub type StandardPubNub = crate::pubnub::PubNub<self::DefaultTransport, self::DefaultRuntime>;
}

pub use default::*;

mod adapters;
mod channel;
mod error;
mod message;
mod mvec;
mod pipe;
mod pubnub;
mod runtime;
mod subscribe;
mod transport;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::{ListenerType, PipeMessage};
    use futures_util::future::FutureExt;
    use futures_util::stream::{FuturesUnordered, StreamExt};
    use json::JsonValue;
    use log::debug;
    use randomize::PCG32;
    use tokio::runtime::current_thread::Runtime;

    const NOV_14_2019: u64 = 15_736_896_000_000_000;
    const NOV_14_2120: u64 = 47_609_856_000_000_000; // TODO: Update this in 100 years

    fn init() {
        let env = env_logger::Env::default().default_filter_or("pubnub=trace");
        let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
    }

    async fn subscribe_loop_exit(pubnub: &StandardPubNub) {
        let mut guard = pubnub.pipe.lock().await;
        match guard.as_mut().unwrap().rx.next().await {
            Some(PipeMessage::Exit) => (),
            error => panic!("Unexpected message: {:?}", error),
        }
    }

    /// Generate a pseudorandom seed for the PRNG.
    fn generate_seed() -> (u64, u64) {
        use byteorder::{ByteOrder, NativeEndian};
        use getrandom::getrandom;

        let mut seed = [0_u8; 16];

        getrandom(&mut seed).expect("failed to getrandom");

        (
            NativeEndian::read_u64(&seed[0..8]),
            NativeEndian::read_u64(&seed[8..16]),
        )
    }

    /// Randomly shuffle a vector of anything, leaving the original vector empty
    fn shuffle<T>(prng: &mut PCG32, list: &mut Vec<T>) -> Vec<T> {
        let length = list.len();
        let mut output = Vec::with_capacity(length);

        for _ in 0..length {
            let length = list.len() - 1;

            let item = if length > 0 {
                list.swap_remove(prng.next_u32() as usize % length)
            } else {
                list.remove(0)
            };

            output.push(item);
        }

        output
    }

    #[test]
    fn pubnub_subscribe_ok() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let agent = "Rust-Agent-Test";

            let mut pubnub = PubNubBuilder::new(publish_key, subscribe_key)
                .agent(agent)
                .build();

            {
                // Create a subscription
                let mut subscription = pubnub.subscribe(channel).await;
                assert_eq!(
                    subscription.name,
                    ListenerType::Channel(channel.to_string())
                );

                // Send a message to it
                let message = JsonValue::String("Hello, world!".to_string());
                debug!("Publishing...");
                let status = pubnub.publish(channel, message).await;
                assert!(status.is_ok());

                // Receive the message
                debug!("Waiting for message...");
                let message = subscription.next().await;
                assert!(message.is_some());

                // Check the message contents
                let message = message.unwrap();
                assert_eq!(message.message_type, Type::Publish);
                let expected = JsonValue::String("Hello, world!".to_string());
                assert_eq!(message.json, expected);
                assert!(message.timetoken.t > NOV_14_2019);
                assert!(message.timetoken.t < NOV_14_2120); // TODO: Update this in 100 years

                debug!("Going to drop Subscription...");
            }
            debug!("Subscription should have been dropped by now");

            debug!("SubscribeLoop should be stopping...");
            subscribe_loop_exit(&pubnub).await;

            debug!("SubscribeLoop should have stopped by now");
        });
    }

    #[test]
    fn pubnub_subscribeloop_drop() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let mut pubnub = PubNub::new(publish_key, subscribe_key);

            {
                // Create a bunch of subscriptions
                let _sub0 = pubnub.subscribe(channel).await;
                let _sub1 = pubnub.subscribe(channel).await;
                let _sub2 = pubnub.subscribe(channel).await;
                let _sub3 = pubnub.subscribe(channel).await;
                let _sub4 = pubnub.subscribe(channel).await;
                let _sub5 = pubnub.subscribe(channel).await;
                let _sub6 = pubnub.subscribe(channel).await;
                let _sub7 = pubnub.subscribe(channel).await;
                let _sub8 = pubnub.subscribe(channel).await;
                let _sub9 = pubnub.subscribe(channel).await;
                let _sub10 = pubnub.subscribe(channel).await;
                let _sub11 = pubnub.subscribe(channel).await;

                // HA-HAAAA! Now we drop 12 at once and see if the `Drop` impl hangs!
            }

            debug!("SubscribeLoop should be stopping...");
            subscribe_loop_exit(&pubnub).await;

            debug!("SubscribeLoop should have stopped by now");
        });
    }

    #[test]
    fn pubnub_subscribeloop_recreate() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo2";

            let mut pubnub = PubNub::new(publish_key, subscribe_key);

            // Create two subscribe loops, dropping each
            {
                let _ = pubnub.subscribe(channel).await;
            }
            subscribe_loop_exit(&pubnub).await;

            {
                let _ = pubnub.subscribe(channel).await;
            }
            subscribe_loop_exit(&pubnub).await;
        });
    }

    #[test]
    fn pubnub_subscribe_clone_ok() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let seed = generate_seed();
            let mut prng = PCG32::seed(seed.0, seed.1);
            let mut streams = Vec::new();

            // Create a client and immediately subscribe
            let mut pubnub1 = PubNub::new("demo", "demo");
            streams.push(pubnub1.subscribe("channel1").await);

            // Create a cloned client and immediate subscribe
            let mut pubnub2 = pubnub1.clone();
            streams.push(pubnub2.subscribe("channel2").await);

            // Subscribe to two more channels from each clone
            streams.push(pubnub1.subscribe("channel3").await);
            streams.push(pubnub2.subscribe("channel4").await);

            // Create a list of publish futures, mix-and-match clients
            let mut publishers = vec![
                pubnub1.publish("channel1", JsonValue::String("test1".to_string())),
                pubnub2.publish("channel2", JsonValue::String("test2".to_string())),
                pubnub2.publish("channel3", JsonValue::String("test3".to_string())),
                pubnub1.publish("channel4", JsonValue::String("test4".to_string())),
            ];

            // Randomly shuffle the list of publishers into a FuturesUnordered
            let mut publishers: FuturesUnordered<_> =
                shuffle(&mut prng, &mut publishers).drain(..).collect();

            // Await all publish futures
            for _ in 0..publishers.len() {
                assert!(publishers.next().await.is_some());
            }
            assert!(publishers.next().await.is_none());

            // Check the streams!
            for (i, stream) in streams.iter_mut().enumerate() {
                let expected = JsonValue::String(format!("test{}", i + 1).to_string());
                assert_eq!(stream.next().await.unwrap().json, expected);
            }

            // Drop the streams
            std::mem::drop(streams);

            // Wait for the subscribe loop to exit
            subscribe_loop_exit(&pubnub1).await;

            // Awaiting the subscribe loop exit on the cloned client should fail
            let future = std::panic::AssertUnwindSafe(subscribe_loop_exit(&pubnub2));
            let result = future.catch_unwind().await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn pubnub_publish_ok() {
        init();

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let publish_key = "demo";
            let subscribe_key = "demo";
            let channel = "demo";

            let agent = "Rust-Agent-Test";

            let pubnub = PubNubBuilder::new(publish_key, subscribe_key)
                .agent(agent)
                .build();

            assert_eq!(pubnub.agent, agent);
            assert_eq!(pubnub.subscribe_key, subscribe_key);
            assert_eq!(pubnub.publish_key, publish_key);

            let message = JsonValue::String("Hi!".to_string());
            let status = pubnub.publish(channel, message).await;
            assert!(status.is_ok());
            let timetoken = status.unwrap();

            assert!(timetoken.t > NOV_14_2019);
            assert!(timetoken.t < NOV_14_2120); // TODO: Update this in 100 years
        });
    }
}
