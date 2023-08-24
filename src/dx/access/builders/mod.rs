//! Access manager builders module.
//!
//! This module contains all builders for the PAM management operations.

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String};

#[doc(inline)]
pub use grant_token::{GrantTokenRequest, GrantTokenRequestBuilder};

#[cfg(not(feature = "serde"))]
#[doc(inline)]
pub use grant_token::GrantTokenRequestWithSerializerBuilder;
pub mod grant_token;

#[doc(inline)]
pub use revoke::{RevokeTokenRequest, RevokeTokenRequestBuilder};
pub mod revoke;

/// Validate [`PubNubClient`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for PAM endpoint usage or not.
pub(in crate::dx::access::builders) fn validate_configuration<T, D>(
    client: &Option<PubNubClientInstance<T, D>>,
) -> Result<(), String> {
    if let Some(client) = client {
        if client.config.subscribe_key.is_empty() {
            return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
        } else if client.config.secret_key.as_deref().unwrap_or("").is_empty() {
            return Err("Incomplete PubNub client configuration: 'secret_key' is empty.".into());
        }
    }

    Ok(())
}
