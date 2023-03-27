//! Deserialization of Rust data structures.
//!
//! This module contains the `Deserialize` trait which is used to implement
//! deserialization of Rust data structures.

use super::PubNubError;

/// Trait for deserializing Rust data structures.
///
/// This trait is used to implement deserialization of Rust data structures.
/// It is used by the [`dx`] modules to deserialize the data returned by the
/// PubNub API.
///
/// To implement this trait, you must provide a `deserialize` method that
/// takes a `&[u8]` and returns a `Result<T, PubNubError>`.
/// If you want to provide your own deserializer, you have to implement this
/// trait over the [`dx`] selected by you in the Cargo.toml file.
///
/// Features and their results:
/// - `publish` - [`PublishResponse`]
///
/// More information about the response of the PubNub API can be found in the
/// [PubNub API Reference](https://www.pubnub.com/docs).
///
/// # Examples
/// ```
/// use pubnub::core::{Deserializer, PubNubError};
/// use pubnub::publish::PublishResult;
///
/// struct MyDeserializer;
///
/// impl Deserializer<PublishResult> for MyDeserializer {
///    fn deserialize(&self, bytes: &[u8]) -> Result<PublishResult, PubNubError> {
///         // ...
///         # unimplemented!()
///    }
/// }
/// ```
///
/// [`dx`]: ../dx/index.html
/// [`PublishResponse`]: ../publish/struct.PublishResponse.html
pub trait Deserializer<T> {
    /// Deserialize a `&[u8]` into a `Result<T, PubNubError>`.
    fn deserialize(&self, bytes: &[u8]) -> Result<T, PubNubError>;
}
