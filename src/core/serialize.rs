//! Serialization modeule
//!
//! This module provides a [`Serialize`] trait for the Pubnub protocol.
//!
//! You can implement this trait for your own types, or use the provided
//! implementations for [`Into<Vec<u8>>`].
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
/// ```no_run // depends on implementation of Serialize
/// use pubnub::Serialize;
///
/// let bytes = vec![1, 2, 3];
/// assert_eq!(bytes.serialize().unwrap(), vec![1, 2, 3]);
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

#[cfg(not(feature = "serde"))]
impl<I> Serialize for I
where
    I: Into<Vec<u8>>,
{
    fn serialize(self) -> Result<Vec<u8>, PubNubError> {
        Ok(self.into())
    }
}

#[cfg(all(test, not(feature = "serde")))]
mod should {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![1, 2, 3] => vec![1, 2, 3]; "vector of bytes")]
    #[test_case("abc" => vec![97, 98, 99]; "string slice")]
    #[test_case("abc".to_string() => vec![97, 98, 99]; "string")]
    fn serialize_vector_of_bytes(input: impl Serialize) -> Vec<u8> {
        input.serialize().unwrap()
    }
}
