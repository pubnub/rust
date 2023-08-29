//! Deserialization of Rust data structures.
//!
//! This module contains the `Deserialize` trait which is used to implement
//! deserialization of Rust data structures.

use crate::core::PubNubError;

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
/// - `publish` - [`PublishResponseBody`]
/// - `access` - [`GrantTokenResponseBody`] and [`RevokeTokenResponseBody`]
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
/// impl Deserializer for MyDeserializer {
///    fn deserialize<PublishResult>(&self, bytes: &[u8]) -> Result<PublishResult, PubNubError> {
///         // ...
///         # unimplemented!()
///    }
/// }
/// ```
///
/// [`dx`]: ../dx/index.html
/// [`PublishResponseBody`]: ../../dx/publish/result/enum.PublishResponseBody.html
/// [`GrantTokenResponseBody`]: ../../dx/access/result/enum.GrantTokenResponseBody.html
/// [`RevokeTokenResponseBody`]: ../../dx/access/result/enum.RevokeTokenResponseBody.html
pub trait Deserializer: Send + Sync {
    /// Deserialize a `&Vec<u8>` into a `Result<T, PubNubError>`.
    ///
    /// # Errors
    ///
    /// This method should return [`PubNubError::DeserializationError`] if the
    /// deserialization fails.
    ///
    /// [`PubNubError::DeserializationError`]: ../enum.PubNubError.html#variant.DeserializationError
    #[cfg(not(feature = "serde"))]
    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, PubNubError>
    where
        T: for<'de> crate::core::Deserialize<'de>;

    /// Deserialize a `&Vec<u8>` into a `Result<T, PubNubError>`.
    ///
    /// # Errors
    ///
    /// This method should return [`PubNubError::DeserializationError`] if the
    /// deserialization fails.
    ///
    /// [`PubNubError::DeserializationError`]: ../enum.PubNubError.html#variant.DeserializationError
    #[cfg(feature = "serde")]
    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, PubNubError>
    where
        T: for<'de> serde::Deserialize<'de>;
}
