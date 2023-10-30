//! # Cryptor module
//!
//! This module contains the [`Cryptor`] trait, which is used to implement
//! crypto algorithms that should be used with [`CryptorProvider`]
//! implementation for data _encryption_ and _decryption_.

use crate::{
    core::PubNubError,
    lib::{alloc::vec::Vec, core::fmt::Debug},
};

/// Encrypted data representation object.
///
/// Objects contain both encrypted data and additional data created by cryptor
/// that will be required to decrypt the data.
#[derive(Debug)]
pub struct EncryptedData {
    /// Cryptor-defined information.
    ///
    /// Cryptor may provide here any information which will be useful when data
    /// should be decrypted.
    ///
    /// For example `metadata` may contain:
    /// * initialization vector
    /// * cipher key identifier
    /// * encrypted `data` length.
    pub metadata: Option<Vec<u8>>,

    /// Encrypted data.
    pub data: Vec<u8>,
}

/// Cryptor trait.
///
/// Types that implement this trait can be used to configure [`CryptoProvider`]
/// implementations for standalone usage or as part of [`PubNubClientInstance`]
/// for automated data _encryption_ and _decryption_.
///
/// To implement this trait, you must provide `encrypt` and `decrypt` methods
/// that takes a `Vec<u8>` and returns a `Result<EncryptedData, PubNubError>`.
///
/// You can implement this trait for your own types, or use one of the provided
/// features to use a `crypto` library.
///
/// You can implement this trait for your cryptor types, or use one of the
/// implementations provided by `crypto` feature.
/// When you implement your cryptor for custom encryption and use multiple
/// platforms, make sure that the same logic is implemented for other SDKs.
///
/// # Examples
/// ```
/// use pubnub::core::{Cryptor, EncryptedData, error::PubNubError};
///
/// #[derive(Debug)]
/// struct MyCryptor;
///
/// impl Cryptor for MyCryptor {
///     fn identifier(&self) -> [u8; 4] {
///         *b"MCID"
///     }
///
///     fn encrypt(&self, source: Vec<u8>) -> Result<EncryptedData, PubNubError> {
///         // Encrypt provided data here
///         Ok(EncryptedData {
///             metadata: None,
///             data: vec![]
///         })
///     }
///
///     fn decrypt(&self, source: EncryptedData) -> Result<Vec<u8>, PubNubError> {
///         // Decrypt provided data here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait Cryptor: Debug + Send + Sync {
    /// Unique cryptor identifier.
    ///
    /// Identifier will be encoded into cryptor data header and passed along
    /// with encrypted data.
    ///
    /// The identifier **must** be 4 bytes long.
    fn identifier(&self) -> [u8; 4];

    /// Encrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::Encryption`] if provided data can't be
    /// _encrypted_ or underlying cryptor misconfigured.
    fn encrypt(&self, data: Vec<u8>) -> Result<EncryptedData, PubNubError>;

    /// Decrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::Decryption`] if provided data can't be
    /// _decrypted_ or underlying cryptor misconfigured.
    fn decrypt(&self, data: EncryptedData) -> Result<Vec<u8>, PubNubError>;
}
