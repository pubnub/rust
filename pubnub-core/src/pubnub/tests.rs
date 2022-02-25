#![cfg(feature = "mock")]

use crate::builder::Builder;
use crate::data::timetoken::Timetoken;
use crate::mock::runtime::MockRuntime;
use crate::mock::transport::{MockTransport, MockTransportError};
use futures_channel::{mpsc, oneshot};
use futures_executor::{block_on, LocalPool};
use futures_util::stream::StreamExt;
use futures_util::task::{LocalSpawnExt, SpawnExt};

use mockall::predicate::eq;
use mockall::Sequence;

use crate::data::message::{self, Message};
use crate::data::{channel, pubsub, request, response};
use crate::json::object;

fn init() {
    pubnub_test_util::init_log();
}

#[test]
fn mocked_pubnub_publish_ok() {
    init();
    block_on(async {
        let mut mock_transport = MockTransport::new();
        let mock_runtime = MockRuntime::new();

        let message = object! {
            "test" => "value",
        };

        mock_transport
            .expect_call::<request::Publish, response::Publish>()
            .with(eq(request::Publish {
                channel: "test_channel".parse().unwrap(),
                payload: message.clone(),
                meta: None,
            }))
            .returning(|_| Box::pin(async { Ok(Timetoken { t: 123, r: 456 }) }));

        let pubnub = Builder::with_components(mock_transport, mock_runtime).build();

        let timetoken = pubnub
            .publish("test_channel".parse().unwrap(), message)
            .await
            .expect("unexpected failure");
        assert_eq!(timetoken.t, 123);
        assert_eq!(timetoken.r, 456);
    });
}

#[test]
fn mocked_pubnub_subscribe_ok() {
    init();
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    let spawner1 = spawner.clone();
    let spawner2 = spawner.clone();
    spawner
        .spawn_local(async {
            // Setup.

            let test_channel: channel::Name = "test_channel".parse().unwrap();

            let (sub_drop_req_tx, sub_drop_req_rx) = oneshot::channel::<()>();
            let (sub_drop_done_tx, sub_drop_done_rx) = oneshot::channel::<()>();
            let (sub_loop_exit_tx, mut sub_loop_exit_rx) = mpsc::channel::<()>(1);

            let messages = vec![Message {
                message_type: message::Type::Publish,
                route: None,
                channel: test_channel.clone(),
                json: object! {
                    "test" => "value",
                },
                timetoken: Timetoken { t: 100, r: 12 },
                client: None,
                subscribe_key: "test_subscribe_key".to_owned(),
                flags: 514,
                ..Message::default()
            }];

            let mut seq = Sequence::new();

            let mock_transport = {
                let mut mock = MockTransport::new();

                let test_channel = test_channel.clone();
                mock.expect_clone()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move || {
                        let mut mock = MockTransport::new();

                        mock.expect_call::<request::Subscribe, response::Subscribe>()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                                timetoken: Timetoken::default(),
                                heartbeat: None,
                            }))
                            .return_once(move |_| {
                                Box::pin(async move {
                                    Ok((messages.clone(), Timetoken { t: 150, r: 1 }))
                                })
                            });

                        mock.expect_call::<request::Subscribe, response::Subscribe>()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                                timetoken: Timetoken { t: 150, r: 1 },
                                heartbeat: None,
                            }))
                            .return_once(move |_| {
                                Box::pin(async move {
                                    // Request drop.
                                    sub_drop_req_tx.send(()).unwrap();

                                    // Wait for the drop to complete.
                                    sub_drop_done_rx.await.unwrap();
                                    unreachable!();
                                })
                            });

                        mock
                    });

                mock
            };

            let mock_runtime = {
                let mut mock = MockRuntime::new();
                mock.expect_mock_workaround_spawn::<()>()
                    .returning_st(move |future| {
                        spawner1.spawn(future).unwrap();
                    });
                mock.expect_clone().times(1).return_once_st(move || {
                    // We got cloned, that has to be subscription's runtime
                    // clone.
                    let mut mock = MockRuntime::new();

                    mock.expect_mock_workaround_spawn::<()>()
                        .returning_st(move |future| {
                            spawner2.spawn(future).unwrap();
                        });

                    mock
                });
                mock
            };

            // Invocations.

            let mut pubnub = Builder::with_components(mock_transport, mock_runtime)
                .subscribe_loop_exit_tx(sub_loop_exit_tx)
                .build();

            let mut subscription = pubnub.subscribe(test_channel.clone()).await;

            let message = subscription.next().await;
            // We got the message we expected to get.
            assert!(message.is_some());

            // Wait for the drop request.
            sub_drop_req_rx.await.unwrap();

            // Drop the subscription, which will cause loop termination.
            drop(subscription);

            // Wait for the loop termination.
            sub_loop_exit_rx.next().await.unwrap();

            // Notify that we've completed with the drop request.
            // Since the loop is now dead, and we were locked on `sub_drop_done_rx`
            // in the response future, this send *has to fail* send error, cause
            // loop termination dropped the response future and the
            // `sub_drop_done_rx` with it (cuase response future owned
            // `sub_drop_done_rx` afetr we moved it).
            sub_drop_done_tx.send(()).unwrap_err();
        })
        .unwrap();

    pool.run();
}

#[allow(clippy::too_many_lines)]
#[test]
fn mocked_pubnub_subscribe_trasport_error_does_not_stall_loop() {
    init();
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    let spawner1 = spawner.clone();
    let spawner2 = spawner.clone();
    spawner
        .spawn_local(async {
            // Setup.

            let test_channel: channel::Name = "test_channel".parse().unwrap();

            let (sub_drop_req_tx, sub_drop_req_rx) = oneshot::channel::<()>();
            let (sub_drop_done_tx, sub_drop_done_rx) = oneshot::channel::<()>();
            let (sub_loop_exit_tx, mut sub_loop_exit_rx) = mpsc::channel::<()>(1);

            let messages = vec![Message {
                message_type: message::Type::Publish,
                route: None,
                channel: test_channel.clone(),
                json: object! {
                    "test" => "value",
                },
                timetoken: Timetoken { t: 100, r: 12 },
                client: None,
                subscribe_key: "test_subscribe_key".to_owned(),
                flags: 514,
                ..Message::default()
            }];

            let mut seq = Sequence::new();

            let mock_transport = {
                let mut mock = MockTransport::new();

                let test_channel = test_channel.clone();
                mock.expect_clone()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move || {
                        let mut mock = MockTransport::new();

                        mock.expect_call::<request::Subscribe, response::Subscribe>()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                                timetoken: Timetoken::default(),
                                heartbeat: None,
                            }))
                            .return_once(move |_| {
                                Box::pin(async move {
                                    Ok((messages.clone(), Timetoken { t: 150, r: 1 }))
                                })
                            });

                        mock.expect_call::<request::Subscribe, response::Subscribe>()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                                timetoken: Timetoken { t: 150, r: 1 },
                                heartbeat: None,
                            }))
                            .return_once(move |_| Box::pin(async move { Err(MockTransportError) }));

                        mock.expect_call::<request::Subscribe, response::Subscribe>()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                to: vec![pubsub::SubscribeTo::Channel(test_channel.clone())],
                                timetoken: Timetoken { t: 150, r: 1 },
                                heartbeat: None,
                            }))
                            .return_once(move |_| {
                                Box::pin(async move {
                                    // Request drop.
                                    sub_drop_req_tx.send(()).unwrap();

                                    // Wait for the drop to complete.
                                    sub_drop_done_rx.await.unwrap();
                                    unreachable!();
                                })
                            });

                        mock
                    });

                mock
            };

            let mock_runtime = {
                let mut mock = MockRuntime::new();
                mock.expect_mock_workaround_spawn::<()>()
                    .returning_st(move |future| {
                        spawner1.spawn(future).unwrap();
                    });
                mock.expect_clone().times(1).return_once_st(move || {
                    // We got cloned, that has to be subscription's runtime
                    // clone.
                    let mut mock = MockRuntime::new();

                    mock.expect_mock_workaround_spawn::<()>()
                        .returning_st(move |future| {
                            spawner2.spawn(future).unwrap();
                        });

                    mock
                });
                mock
            };

            // Invocations.

            let mut pubnub = Builder::with_components(mock_transport, mock_runtime)
                .subscribe_loop_exit_tx(sub_loop_exit_tx)
                .build();

            let mut subscription = pubnub.subscribe(test_channel.clone()).await;

            let message = subscription.next().await;
            // We got the message we expected to get.
            assert!(message.is_some());

            // Wait for the drop request.
            sub_drop_req_rx.await.unwrap();

            // Drop the subscription, which will cause loop termination.
            drop(subscription);

            // Wait for the loop termination.
            sub_loop_exit_rx.next().await.unwrap();

            // Notify that we've completed with the drop request.
            // Since the loop is now dead, and we were locked on `sub_drop_done_rx`
            // in the response future, this send *has to fail* send error, cause
            // loop termination dropped the response future and the
            // `sub_drop_done_rx` with it (cuase response future owned
            // `sub_drop_done_rx` afetr we moved it).
            sub_drop_done_tx.send(()).unwrap_err();
        })
        .unwrap();

    pool.run();
}
