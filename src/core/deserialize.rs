//! Deserialization module
//!
//! This module provides a [`Deserialize`] trait for the Pubnub protocol.
//!
//! You can implement this trait for your own types, or use one of the provided
//! features to use a deserialization library.
//!
//! [`Deserialize`]: trait.Deserialize.html

use crate::core::PubNubError;

/// Deserialize values
///
/// This trait provides a [`deserialize`] method for the Pubnub protocol.
///
/// You can implement this trait for your own types, or use the provided
/// implementations for [`Into<Vec<u8>>`].
///
/// [`deserialize`]: #tymethod.deserialize
pub trait Deserialize<'de>: Send + Sync {
    /// Type to which binary data should be mapped.
    type Type;

    /// Deserialize the value
    fn deserialize(bytes: &'de [u8]) -> Result<Self::Type, PubNubError>;
}
