//! TODO: docs

pub mod publish;

pub use pubnub_client::PubNubClient;
pub mod pubnub_client;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
