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
//! use json::object;
//! use pubnub::PubNub;
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
//! # Ok::<(), pubnub::Error>(())
//! # };
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]

pub use crate::error::Error;
pub use crate::message::{Message, Timetoken, Type};
pub use crate::pubnub::{PubNub, PubNubBuilder};
pub use crate::subscribe::Subscription;
pub use json::JsonValue;

mod channel;
mod error;
mod http;
mod message;
mod mvec;
mod pipe;
mod pubnub;
mod subscribe;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::{ListenerType, PipeMessage};
    use futures_util::stream::StreamExt;
    use log::debug;
    use tokio::runtime::current_thread::Runtime;

    fn init() {
        let env = env_logger::Env::default().default_filter_or("pubnub=trace");
        let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
    }

    async fn subscribe_loop_exit(pubnub: &PubNub) {
        let mut guard = pubnub.pipe.lock().unwrap();
        match guard.as_mut().unwrap().rx.next().await {
            Some(PipeMessage::Exit) => (),
            error => panic!("Unexpected message: {:?}", error),
        }
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
                assert_eq!(message.timetoken.t.len(), 17);
                assert!(message.timetoken.t.chars().all(|c| c >= '0' && c <= '9'));

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

            assert_eq!(timetoken.t.len(), 17);
            assert!(timetoken.t.chars().all(|c| c >= '0' && c <= '9'));
        });
    }
}
