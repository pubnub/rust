//! Serialization modeule
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
/// [`serialize`]: #tymethod.serialize
///
/// # Examples
/// ```no_run
/// use pubnub::Serialize;
///
/// struct Foo {
///   bar: String,
/// }
///  
/// impl Serialize for Foo {
///   fn serialize(self) -> Result<Vec<u8>, pubnub::PubNubError> {
///     Ok(format!("{{\"bar\":\"{}\"}}", self.bar).into_bytes())
///   }
/// }
///
/// let bytes = Foo { bar: "baz".into() };
/// assert_eq!(bytes.serialize().unwrap(), b"{\"bar\":\"baz\"}".to_vec());
/// ```
pub trait Serialize {
    /// Serialize the value
    ///
    /// # Errors
    /// Should return an [`PubNubError::SerializeError`] if the value cannot be serialized.
    ///
    /// [`PubNubError::SerializeError`]: ../error/enum.PubNubError.html#variant.SerializeError
    ///
    /// # Examples
    /// ```
    /// use pubnub::Serialize;
    ///
    /// struct Foo;
    ///
    /// impl Serialize for Foo {
    ///    fn serialize(self) -> Result<Vec<u8>, pubnub::PubNubError> {
    ///         Ok(vec![1, 2, 3])
    ///    }
    /// }
    ///```
    fn serialize(self) -> Result<Vec<u8>, PubNubError>;
}
