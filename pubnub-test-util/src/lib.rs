//! Test utilities used in PubNub crates suite.

#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

/// Initialize the logger.
///
/// Takes the value of `TEST_LOG` env var, uses `pubnub=trace` by default.
/// Initializes `env_logger` in test mode.
pub fn init_log() {
    let val = std::env::var("TEST_LOG").unwrap_or_else(|_| "pubnub=trace".to_owned());
    let env = env_logger::Env::default().default_filter_or(val);
    let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
}
