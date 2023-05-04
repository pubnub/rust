//! Serde implementation for PubNub [`Deserializer`] trait.
//!
//! This module provides a `serde` deserializer for the Pubnub protocol.
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

use crate::{
    core::{Deserializer, PubNubError},
    lib::a::string::ToString,
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
pub struct SerdeDeserializer;

impl<'de, T> Deserializer<'de, T> for SerdeDeserializer
where
    T: serde::Deserialize<'de>,
{
    fn deserialize(&self, bytes: &'de [u8]) -> Result<T, crate::core::PubNubError> {
        serde_json::from_slice(bytes).map_err(|e| PubNubError::Deserialization {
            details: e.to_string(),
        })
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::lib::a::string::String;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Foo {
        bar: String,
    }

    #[test]
    fn deserialize() {
        let sut = SerdeDeserializer;

        let result: Foo = sut.deserialize(b"{\"bar\":\"baz\"}").unwrap();

        assert_eq!(
            result,
            Foo {
                bar: "baz".to_string()
            }
        );
    }
}
