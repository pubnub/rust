//! Serialization module
//!
//! This module provides a [`Serialize`] trait for the Pubnub protocol.
//!
//! You can implement this trait for your own types, or use one of the provided
//! features to use a serialization library.
//!
//! [`Serialize`]: trait.Serialize.html

use super::PubNubError;

/// Serialize values
///
/// This trait provides a [`serialize`] method for the Pubnub protocol.
///
/// You can implement this trait for your own types, or use the provided
/// implementations for [`Into<Vec<u8>>`].
///
/// # Examples
/// ```no_run
/// use pubnub::core::{Serialize, PubNubError};
///
/// struct Foo {
///   bar: String,
/// }
///  
/// impl Serialize for Foo {
///   fn serialize(self) -> Result<Vec<u8>, PubNubError> {
///     Ok(format!("{{\"bar\":\"{}\"}}", self.bar).into_bytes())
///   }
/// }
///
/// let bytes = Foo { bar: "baz".into() };
/// assert_eq!(bytes.serialize().unwrap(), b"{\"bar\":\"baz\"}".to_vec());
/// ```
///
/// [`serialize`]: #tymethod.serialize
pub trait Serialize {
    /// Serialize the value
    ///
    /// # Errors
    /// Should return an [`PubNubError::SerializeError`] if the value cannot be serialized.
    ///
    /// # Examples
    /// ```
    /// use pubnub::core::{Serialize, PubNubError};
    ///
    /// struct Foo;
    ///
    /// impl Serialize for Foo {
    ///    fn serialize(self) -> Result<Vec<u8>, PubNubError> {
    ///         Ok(vec![1, 2, 3])
    ///    }
    /// }
    /// ```
    ///
    /// [`PubNubError::SerializeError`]: ../error/enum.PubNubError.html#variant.SerializeError
    fn serialize(self) -> Result<Vec<u8>, PubNubError<'static>>;
}
