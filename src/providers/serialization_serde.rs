//! Serde implementation for PubNub [`Serialize`] trait.
//!
//! This module provides a `serde` serializer for the Pubnub protocol.
//!
//! # Examples
//! ```
//! use pubnub::core::Serialize as _;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Foo {
//!    bar: String,
//! }
//!
//! let foo = Foo { bar: "baz".to_string() };
//! assert_eq!(foo.serialize().unwrap(), b"{\"bar\":\"baz\"}".to_vec());
//! ```
//!
//! [`Serialize`]: ../trait.Serialize.html
impl<S> crate::core::Serialize for S
where
    S: serde::Serialize,
{
    fn serialize(self) -> Result<Vec<u8>, crate::core::PubNubError> {
        serde_json::to_vec(&self)
            .map_err(|e| crate::core::PubNubError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod should {
    use crate::core::Serialize;

    #[test]
    fn serialize_serde_values() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Foo {
            bar: String,
        }

        let sut = Foo { bar: "baz".into() };
        assert_eq!(sut.serialize().unwrap(), b"{\"bar\":\"baz\"}".to_vec());
    }
}
