//! Serde implementation for PubNub [`Deserializer`] trait.
//!
//! This module provides a `serde` deserializer for the Pubnub protocol.
//!
//! # Examples
//! ```
//! use pubnub::core::Serialize as _;
//!
//! #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
//! struct Foo {
//!    bar: String,
//! }
//!
//! let foo = Foo { bar: "baz".to_string() };
//! assert_eq!(foo.serialize().unwrap(), b"{\"bar\":\"baz\"}".to_vec());
//! ```
//!
//! [`Serialize`]: ../trait.Serialize.html

use crate::{
    core::{Deserializer, PubNubError},
    lib::alloc::string::ToString,
};

/// Serde implementation for PubNub [`Deserializer`] trait.
///
/// This struct implements the [`Deserializer`] trait for the [`serde`] crate.
/// It is used by the [`dx`] modules to deserialize the data returned by the
/// PubNub API.
///
/// [`Deserializer`]: ../trait.Deserializer.html
/// [`serde`]: https://crates.io/crates/serde
/// [`dx`]: ../dx/index.html
#[derive(Debug, Clone)]
pub struct DeserializerSerde;

impl Deserializer for DeserializerSerde {
    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, PubNubError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_slice(bytes).map_err(|e| PubNubError::Deserialization {
            details: e.to_string(),
        })
    }
}

impl<'de, D> crate::core::Deserialize<'de> for D
where
    D: serde::Deserialize<'de> + Send + Sync,
{
    type Type = D;

    fn deserialize(bytes: &'de [u8]) -> Result<Self::Type, PubNubError> {
        serde_json::from_slice(bytes).map_err(|e| PubNubError::Deserialization {
            details: e.to_string(),
        })
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::lib::alloc::string::String;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Foo {
        bar: String,
    }

    #[test]
    fn deserialize() {
        let sut = DeserializerSerde;

        let result: Foo = sut.deserialize(&Vec::from("{\"bar\":\"baz\"}")).unwrap();

        assert_eq!(
            result,
            Foo {
                bar: "baz".to_string()
            }
        );
    }
}
