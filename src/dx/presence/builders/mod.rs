//! # Presence API request builders.

use crate::dx::pubnub_client::PubNubClientInstance;

pub(crate) mod heartbeat;

/// Validate [`PubNubClient`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for presence endpoint usage or not.
pub(in crate::dx::presence::builders) fn validate_configuration<T>(
    client: &Option<PubNubClientInstance<T>>,
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
