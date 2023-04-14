//! Serialization of Rust data structures.
//!
//! This module contains the `Serialize` trait which is used to implement
//! serialization of Rust data structures.

use super::PubNubError;

/// Trait for serializing Rust data structures.
///
/// This trait is used to implement serialization of Rust data structures.
/// It is used by the [`dx`] modules to serialize the data sent to PubNub API.
///
/// To implement this trait, you must provide a `serialize` method that
/// takes a `&T` and returns a `Result<Vec<u8>, PubNubError>`.
/// If you want to provide your own serializer, you have to implement this
/// trait over the [`dx`] selected by you in the Cargo.toml file.
///
/// Features and their results:
/// - `publish` - [`PublishResponse`]
/// - `grant_token` - [`GrantTokenResponse`]
///
/// More information about the response of the PubNub API can be found in the
/// [PubNub API Reference](https://www.pubnub.com/docs).
///
/// # Examples
/// ```no_run
/// use pubnub::core::{Deserializer, PubNubError};
/// use pubnub::publish::PublishResult;
///
/// struct MyDeserializer;
///
/// impl<'de> Deserializer<'de, PublishResult> for MyDeserializer {
///    fn deserialize(&self, bytes: &'de [u8]) -> Result<PublishResult, PubNubError> {
///         // ...
///         # unimplemented!()
///    }
/// }
/// ```
///
/// [`dx`]: ../dx/index.html
pub trait Serializer<'se, T> {
    /// Serialize a `&T` into a `Result<Vec<u8>, PubNubError>`.
    ///
    /// # Errors
    ///
    /// This method should return [`PubNubError::Serialization`] if the
    /// serialization fails.
    ///
    /// [`PubNubError::Serialization`]: ../enum.PubNubError.html#variant.Serialization
    fn serialize(&self, object: &'se T) -> Result<Vec<u8>, PubNubError>;
}
