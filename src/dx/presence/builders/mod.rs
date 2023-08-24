//! # Presence API request builders.

#[doc(inline)]
pub use heartbeat::{HeartbeatRequest, HeartbeatRequestBuilder};
pub mod heartbeat;

#[doc(inline)]
pub use set_state::{SetStateRequest, SetStateRequestBuilder};
pub mod set_state;

#[doc(inline)]
pub use leave::{LeaveRequest, LeaveRequestBuilder};
pub mod leave;

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String};

/// Validate [`PubNubClient`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for presence endpoint usage or not.
pub(in crate::dx::presence::builders) fn validate_configuration<T, D>(
    client: &Option<PubNubClientInstance<T, D>>,
) -> Result<(), String> {
    if let Some(client) = client {
        if client.config.subscribe_key.is_empty() {
            return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
        }
    }

    Ok(())
}
