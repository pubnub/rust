//! Subscribe builders module.

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String};

#[cfg(not(feature = "serde"))]
#[doc(inline)]
pub use subscribe::SubscribeRequestWithDeserializerBuilder;

#[doc(inline)]
pub use subscribe::{SubscribeRequest, SubscribeRequestBuilder};
pub mod subscribe;

pub use subscription::{SubscriptionBuilder, SubscriptionBuilderError};
pub mod subscription;

/// Validate [`PubNubClientInstance`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for subscribe / unsubscribe endpoint usage or not.
pub(in crate::dx::subscribe::builders) fn validate_configuration<T>(
    client: &Option<PubNubClientInstance<T>>,
) -> Result<(), String> {
    if let Some(client) = client {
        if client.config.subscribe_key.is_empty() {
            return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
        }
    }

    Ok(())
}
