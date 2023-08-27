//! # Presence API builders module.
//!
//! Module contains set fo builders which provide access to [`PubNub`] presence
//! API: [`SetStateRequestBuilder`].
//!
//! [`PubNub`]: https://www.pubnub.com

#[doc(inline)]
pub(crate) use heartbeat::{HeartbeatRequest, HeartbeatRequestBuilder};
pub(crate) mod heartbeat;

#[doc(inline)]
pub use set_state::{SetStateRequest, SetStateRequestBuilder};
pub mod set_state;

#[doc(inline)]
pub(crate) use leave::{LeaveRequest, LeaveRequestBuilder};
pub(crate) mod leave;

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String};

/// Validate [`PubNubClient`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for presence endpoint usage or not.
pub(in crate::dx::presence::builders) fn validate_configuration<T, D>(
    client: &Option<PubNubClientInstance<T, D>>,
) -> Result<(), String> {
    let client = client
        .as_ref()
        .expect("PubNub client instance not set.".into());
    if client.config.subscribe_key.is_empty() {
        return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
    }

    Ok(())
}
