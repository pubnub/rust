use super::*;
use crate::tests::mock::runtime::MockRuntime;
use crate::tests::mock::transport::MockTransport;
use futures_executor::block_on;

use mockall::predicate::*;

use crate::json::object;
use http::Uri;

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
            .expect_sync_publish_request()
            .with(eq(Uri::from_static(
                r#"https://pubnub.test/publish/test_publish_key/test_subscribe_key/0/test%5Fchannel/0/%7B%22test%22%3A%22value%22%7D"#,
            )))
            .returning(|_| Ok(Timetoken { t: 123, r: 456 }));

        let pubnub = PubNubBuilder::with_components(
            "test_publish_key",
            "test_subscribe_key",
            mock_transport,
            mock_runtime,
        )
        .origin("pubnub.test")
        .build();

        let timetoken = pubnub
            .publish("test_channel", message)
            .await
            .expect("unexpected failure");
        assert_eq!(timetoken.t, 123);
        assert_eq!(timetoken.r, 456);
    })
}
