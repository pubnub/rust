//! Presence module.
//!
//! The presence module allows retrieving presence information and managing the
//! state in specific channels associated with specific `uuid`.

use crate::core::Serialize;
use crate::dx::presence::builders::heartbeat::{HeartbeatRequestsBuilder, HeartbeatState};
use crate::dx::pubnub_client::PubNubClientInstance;
use crate::providers::deserialization_serde::SerdeDeserializer;

pub(crate) mod event_engine;

pub mod builders;

#[doc(inline)]
pub mod result;

impl<T> PubNubClientInstance<T> {
    /// Create a heartbeat request builder.
    /// This method is used to announce the presence of `user_id` on the provided list of
    /// channels and/or groups.
    ///
    /// Instance of [`fom`] returned.
    #[cfg(feature = "serde")]
    pub fn heartbeat<U>(&self) -> HeartbeatRequestsBuilder<T, U, SerdeDeserializer>
    where
        U: Serialize + HeartbeatState + Clone,
    {
        HeartbeatRequestsBuilder {
            pubnub_client: Some(self.clone()),
            deserializer: Some(SerdeDeserializer),
            channels: None,
            channel_groups: None,
            state: None,
            heartbeat: Some(self.config),
            user_id: Some(self.config.user_id.clone().to_string()),
        }
    }
}
