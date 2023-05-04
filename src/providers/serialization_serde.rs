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

use crate::lib::{a::string::ToString, Vec};

/// Serde implementation for PubNub [`Serializer`] trait.
///
/// This struct implements the [`Serializer`] trait for the [`serde`] crate.
/// It is used by the [`dx`] modules to serialize the data sent to PubNub API.
///
/// [`serde`]: https://crates.io/crates/serde
/// [`dx`]: ../dx/index.html
/// [`Serializer`]: ../core/trait.Serializer.html
pub struct SerdeSerializer;

impl<'se, T> crate::core::Serializer<'se, T> for SerdeSerializer
where
    T: serde::Serialize,
{
    fn serialize(&self, object: &'se T) -> Result<Vec<u8>, crate::core::PubNubError> {
        serde_json::to_vec(object)
            .map_err(|e| crate::core::PubNubError::Serialization(e.to_string()))
    }
}

impl<S> crate::core::Serialize for S
where
    S: serde::Serialize,
{
    fn serialize(self) -> Result<Vec<u8>, crate::core::PubNubError> {
        serde_json::to_vec(&self).map_err(|e| crate::core::PubNubError::Serialization {
            details: e.to_string(),
        })
    }
}

#[cfg(test)]
mod should {
    use crate::core::Serialize;
    use crate::lib::a::string::String;

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
