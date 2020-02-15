use super::*;
use crate::builder::Builder;
use crate::mock::runtime::MockRuntime;
use crate::mock::transport::MockTransport;
use futures_channel::{mpsc, oneshot};
use futures_executor::{block_on, LocalPool};
use futures_util::stream::StreamExt;
use futures_util::task::{LocalSpawnExt, SpawnExt};

use mockall::predicate::*;
use mockall::Sequence;

use crate::data::message::{Message, Type};
use crate::data::request;
use crate::json::object;

fn init() {
    let env = env_logger::Env::default().default_filter_or("pubnub=trace");
    let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
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
            .expect_mock_workaround_publish_request()
            .with(eq(request::Publish {
                channel: "test_channel".to_string(),
                payload: message.clone(),
                meta: None,
            }))
            .returning(|_| Box::pin(async { Ok(Timetoken { t: 123, r: 456 }) }));

        let pubnub = Builder::with_components(mock_transport, mock_runtime).build();

        let timetoken = pubnub
            .publish("test_channel", message)
            .await
            .expect("unexpected failure");
        assert_eq!(timetoken.t, 123);
        assert_eq!(timetoken.r, 456);
    })
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

            let test_channel = "test_channel";

            let (sub_drop_req_tx, sub_drop_req_rx) = oneshot::channel::<()>();
            let (sub_drop_done_tx, sub_drop_done_rx) = oneshot::channel::<()>();
            let (sub_loop_exit_tx, mut sub_loop_exit_rx) = mpsc::channel::<()>(1);

            let messages = vec![Message {
                message_type: Type::Publish,
                route: Some(test_channel.to_owned()),
                channel: test_channel.to_owned(),
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

                mock.expect_clone()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move || {
                        let mut mock = MockTransport::new();

                        mock.expect_mock_workaround_subscribe_request()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                channels: vec![test_channel.to_owned()],
                                timetoken: Timetoken::default(),
                            }))
                            .return_once(move |_| {
                                Box::pin(async move {
                                    Ok((messages.clone(), Timetoken { t: 150, r: 1 }))
                                })
                            });

                        mock.expect_mock_workaround_subscribe_request()
                            .times(1)
                            .in_sequence(&mut seq)
                            .with(eq(request::Subscribe {
                                channels: vec![test_channel.to_owned()],
                                timetoken: Timetoken { t: 150, r: 1 },
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

            let mut subscription = pubnub.subscribe(test_channel).await;

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

    pool.run()
}
