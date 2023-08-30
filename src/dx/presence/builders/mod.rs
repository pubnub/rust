//! # Presence API builders module.
//!
//! Module contains set fo builders which provide access to [`PubNub`] presence
//! API: [`SetStateRequestBuilder`].
//!
//! [`PubNub`]: https://www.pubnub.com

#[doc(inline)]
pub(crate) use heartbeat::HeartbeatRequestBuilder;
pub(crate) mod heartbeat;

#[doc(inline)]
pub use set_presence_state::{SetStateRequest, SetStateRequestBuilder};
pub mod set_presence_state;

#[doc(inline)]
pub(crate) use leave::LeaveRequestBuilder;
pub(crate) mod leave;

#[doc(inline)]
pub(crate) use here_now::HereNowRequestBuilder;
pub(crate) mod here_now;

#[doc(inline)]
pub(crate) use where_now::WhereNowRequestBuilder;
pub(crate) mod where_now;

#[doc(inline)]
pub(crate) use get_presence_state::{GetStateRequest, GetStateRequestBuilder};
pub(crate) mod get_presence_state;

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
        .unwrap_or_else(|| panic!("PubNub client instance not set."));

    if client.config.subscribe_key.is_empty() {
        return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
    }

    Ok(())
}
