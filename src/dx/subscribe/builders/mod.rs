//! # Subscribe API builders module.
//!
//! Module provides set of builders to work with [`PubNub`] subscribe API:
//! [`SubscribeRequestBuilder`] and [`SubscriptionBuilder`].
//!
//! [`PubNub`]: https://www.pubnub.com

#[doc(inline)]
pub(crate) use subscribe::SubscribeRequestBuilder;
pub(crate) mod subscribe;

#[cfg(feature = "std")]
#[doc(inline)]
pub use subscription::{SubscriptionBuilder, SubscriptionBuilderError};

#[cfg(feature = "std")]
pub mod subscription;

pub mod raw;

use crate::{dx::pubnub_client::PubNubClientInstance, lib::alloc::string::String};

/// Validate [`PubNubClient`] configuration.
///
/// Check whether if the [`PubNubConfig`] contains all the required fields set
/// for subscribe endpoint usage or not.
pub(in crate::dx::subscribe::builders) fn validate_configuration<T, D>(
    client: &Option<PubNubClientInstance<T, D>>,
) -> Result<(), String> {
    if let Some(client) = client {
        if client.config.subscribe_key.is_empty() {
            return Err("Incomplete PubNub client configuration: 'subscribe_key' is empty.".into());
        }
    }

    Ok(())
}
