use futures_channel::mpsc;
use futures_util::stream::{FuturesUnordered, StreamExt};
use json::JsonValue;
use log::debug;
use pubnub_hyper::core::data::channel;
use pubnub_hyper::core::data::message::Type;
use pubnub_hyper::runtime::tokio_global::TokioGlobal;
use pubnub_hyper::transport::hyper::Hyper;
use pubnub_hyper::Builder;
use randomize::PCG32;

mod common;

const NOV_14_2019: u64 = 15_736_896_000_000_000;
const NOV_14_2120: u64 = 47_609_856_000_000_000; // TODO: Update this in 100 years

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
    common::init();
    common::current_thread_block_on(async {
        let channel: channel::Name = "demo2".parse().unwrap();

        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let (subscribe_loop_exit_tx, mut subscribe_loop_exit_rx) = mpsc::channel(1);

        let mut pubnub = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .subscribe_loop_exit_tx(subscribe_loop_exit_tx)
            .build();

        {
            // Create a subscription
            let mut subscription = pubnub.subscribe(channel.clone()).await;

            // Send a message to it
            let message = JsonValue::String("Hello, world!".to_string());
            debug!("Publishing...");
            let status = pubnub.publish(channel.clone(), message).await;
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
        subscribe_loop_exit_rx.next().await;

        debug!("SubscribeLoop should have stopped by now");
    });
}

#[test]
fn pubnub_subscribeloop_drop() {
    common::init();
    common::current_thread_block_on(async {
        let channel: channel::Name = "demo2".parse().unwrap();

        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let (subscribe_loop_exit_tx, mut subscribe_loop_exit_rx) = mpsc::channel(1);

        let mut pubnub = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .subscribe_loop_exit_tx(subscribe_loop_exit_tx)
            .build();

        {
            // Create a bunch of subscriptions
            let _sub0 = pubnub.subscribe(channel.clone()).await;
            let _sub1 = pubnub.subscribe(channel.clone()).await;
            let _sub2 = pubnub.subscribe(channel.clone()).await;
            let _sub3 = pubnub.subscribe(channel.clone()).await;
            let _sub4 = pubnub.subscribe(channel.clone()).await;
            let _sub5 = pubnub.subscribe(channel.clone()).await;
            let _sub6 = pubnub.subscribe(channel.clone()).await;
            let _sub7 = pubnub.subscribe(channel.clone()).await;
            let _sub8 = pubnub.subscribe(channel.clone()).await;
            let _sub9 = pubnub.subscribe(channel.clone()).await;
            let _sub10 = pubnub.subscribe(channel.clone()).await;
            let _sub11 = pubnub.subscribe(channel.clone()).await;

            // HA-HAAAA! Now we drop 12 at once and see if the `Drop` impl hangs!
        }

        debug!("SubscribeLoop should be stopping...");
        subscribe_loop_exit_rx.next().await;

        debug!("SubscribeLoop should have stopped by now");
    });
}

#[test]
fn pubnub_subscribeloop_recreate() {
    common::init();
    common::current_thread_block_on(async {
        let channel: channel::Name = "demo2".parse().unwrap();

        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let (subscribe_loop_exit_tx, mut subscribe_loop_exit_rx) = mpsc::channel(1);

        let mut pubnub = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .subscribe_loop_exit_tx(subscribe_loop_exit_tx)
            .build();

        // Create two subscribe loops, dropping each
        {
            let _ = pubnub.subscribe(channel.clone()).await;
        }
        assert!(subscribe_loop_exit_rx.next().await.is_some());

        {
            let _ = pubnub.subscribe(channel).await;
        }
        assert!(subscribe_loop_exit_rx.next().await.is_some());
    });
}

#[test]
fn pubnub_subscribe_clone_ok() {
    common::init();
    common::current_thread_block_on(async {
        let seed = generate_seed();
        let mut prng = PCG32::seed(seed.0, seed.1);
        let mut streams = Vec::new();

        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let (subscribe_loop_exit_tx, mut subscribe_loop_exit_rx) = mpsc::channel(1);

        // Create a client and immediately subscribe
        let mut pubnub1 = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .subscribe_loop_exit_tx(subscribe_loop_exit_tx)
            .build();
        streams.push(pubnub1.subscribe("channel1".parse().unwrap()).await);

        // Create a cloned client and immediate subscribe
        let mut pubnub2 = pubnub1.clone();
        streams.push(pubnub2.subscribe("channel2".parse().unwrap()).await);

        // Subscribe to two more channels from each clone
        streams.push(pubnub1.subscribe("channel3".parse().unwrap()).await);
        streams.push(pubnub2.subscribe("channel4".parse().unwrap()).await);

        // Create a list of publish futures, mix-and-match clients
        let mut publishers = vec![
            pubnub1.publish(
                "channel1".parse().unwrap(),
                JsonValue::String("test1".to_string()),
            ),
            pubnub2.publish(
                "channel2".parse().unwrap(),
                JsonValue::String("test2".to_string()),
            ),
            pubnub2.publish(
                "channel3".parse().unwrap(),
                JsonValue::String("test3".to_string()),
            ),
            pubnub1.publish(
                "channel4".parse().unwrap(),
                JsonValue::String("test4".to_string()),
            ),
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
        drop(streams);

        // Wait for the subscribe loop to exit.
        subscribe_loop_exit_rx.next().await;

        // Prevent any more messages from being submitted to the channel.
        // If there is another subscribe loop that completes (i.e. from a
        // misbehaving cloned) client, it will panic upon sending an exit
        // signal.
        subscribe_loop_exit_rx.close();

        // Ensure there are no messages left in queue, since otherwise it'd
        // mean that there was more than one event loop.
        assert!(subscribe_loop_exit_rx.next().await.is_none());
    });
}

#[test]
fn pubnub_subscribe_clones_share_loop() {
    common::init();
    common::current_thread_block_on(async {
        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let (subscribe_loop_exit_tx, mut subscribe_loop_exit_rx) = mpsc::channel(1);

        // Create a client, dso not subscribe to avoid bootstrapping the
        // subscribe loop immediately.
        let mut pubnub1 = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .subscribe_loop_exit_tx(subscribe_loop_exit_tx)
            .build();

        // Clone the client.
        let mut pubnub2 = pubnub1.clone();

        // Subscribe to spawn the subscribe loop on the original.
        let sub1 = pubnub1.subscribe("channel1".parse().unwrap()).await;

        // Subscribe to potentially spawn the subscribe loop on the clone.
        let sub2 = pubnub2.subscribe("channel2".parse().unwrap()).await;

        // Dropping `sub1` should not exit the loop if it's shared, but will
        // if the loop is not shared it would exit.
        drop(sub1);

        // At this point, the loop should be still working, because it's suppsed
        // to be shared among the clones, and since there's `sub2` that has to
        // be serviced it can't exit just yet.

        // Dropping `sub2`, this should make the shared subscribe loop exit.
        drop(sub2);

        // Wait for the subscribe loop to exit.
        assert!(subscribe_loop_exit_rx.next().await.is_some());

        // Prevent any more messages from being submitted to the channel.
        // If there is another subscribe loop that completes (i.e. from a
        // misbehaving cloned) client, it will panic upon sending an exit
        // signal.
        subscribe_loop_exit_rx.close();

        // Ensure there are no messages left in queue, since otherwise it'd
        // mean that there was more than one event loop.
        assert!(subscribe_loop_exit_rx.next().await.is_none());
    });
}

#[test]
fn pubnub_publish_ok() {
    common::init();
    common::current_thread_block_on(async {
        let channel = "demo".parse().unwrap();

        let transport = Hyper::new()
            .agent("Rust-Agent-Test")
            .publish_key("demo")
            .subscribe_key("demo")
            .build()
            .unwrap();

        let pubnub = Builder::new()
            .transport(transport)
            .runtime(TokioGlobal)
            .build();

        let message = JsonValue::String("Hi!".to_string());
        let status = pubnub.publish(channel, message).await;
        assert!(status.is_ok());
        let timetoken = status.unwrap();

        assert!(timetoken.t > NOV_14_2019);
        assert!(timetoken.t < NOV_14_2120); // TODO: Update this in 100 years
    });
}
