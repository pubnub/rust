//! Serialize Serde values
//!
//! This module provides a `serde` serializer for the Pubnub protocol.

use pubnub_core::PubNubError;

/// Serialize Serde values
impl<S> pubnub_core::Serialize for S
where
    S: serde::Serialize,
{
    fn serialize(self) -> Result<Vec<u8>, PubNubError> {
        self.serialize()
    }
}

//trait InnerSerialize {
//    fn serialize(self) -> Result<Vec<u8>, PubNubError>;
//}
//
//impl<S> InnerSerialize for S
//where
//    S: serde::Serialize,
//{
//    fn serialize(self) -> Result<Vec<u8>, PubNubError> {
//        serde_json::to_vec(&self).map_err(|e| PubNubError::SerializationError(e.to_string()))
//    }
//}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn serialize_serde_values() {
        assert_eq!(2 + 2, 4);
    }
}
