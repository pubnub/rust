//! Cryptor module
//!
//! This module contains the [`Cryptor`] trait which is used to implement
//! encryption and decryption of published data.

use crate::core::error::PubNubError;

/// This trait is used to encrypt and decrypt messages sent to the
/// [`PubNub API`].
///
/// It is used by the [`dx`] modules to encrypt messages sent to PubNub and
/// returned by the [`PubNub API`].
///
/// To implement this trait, you must provide `encrypt` and `decrypt` methods
/// that takes a `&[u8]` and returns a `Result<Vec<u8>, PubNubError>`.
///
/// You can implement this trait for your own types, or use one of the provided
/// features to use a crypto library.
/// When you use this trait to make your own crypto, make sure that other SDKs
/// use the same encryption and decryption algorithms.
///
/// # Examples
/// ```
/// use pubnub::core::{Cryptor, error::PubNubError};
///
/// struct MyCryptor;
///
/// impl Cryptor for MyCryptor {
///     fn encrypt<'en, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
///     where
///         T: Into<&'en [u8]>
///     {
///         // Encrypt provided data here
///
///         Ok(vec![])
///     }
///
///     fn decrypt<'de, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
///     where
///         T: Into<&'de [u8]>
///     {
///         // Decrypt provided data here
///
///         Ok(vec![])
///     }
/// }
/// ```
///
/// [`dx`]: ../dx/index.html
/// [`PubNub API`]: https://www.pubnub.com/docs
pub trait Cryptor {
    /// Decrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::EncryptionError`] if provided data can't
    /// be encrypted or underlying cryptor misconfigured.
    fn encrypt<'en, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
    where
        T: Into<&'en [u8]>;

    /// Decrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::DecryptionError`] if provided data can't
    /// be decrypted or underlying cryptor misconfigured.
    fn decrypt<'de, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
    where
        T: Into<&'de [u8]>;
}