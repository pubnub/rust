//! Crypto module
//!
//! This module contains a [`CryptoModule`] which allows to handle encrypted
//! data in a backward compatible way. [`AesCbcCryptor`] and [`LegacyCryptor`]
//! are available for [`CryptoModule`] configuration. The built-in modules also
//! decrypt AES-256-GCM (`ACGM`) payloads. AES-256-GCM encryption is not
//! available.

#[doc(inline)]
pub use cryptors::{AesCbcCryptor, LegacyCryptor};
pub mod cryptors;

#[doc(inline)]
pub use crypto_module::CryptoModule;
pub mod crypto_module;

#[doc(inline)]
pub(crate) use cryptor_header::CryptorHeader;
pub(crate) mod cryptor_header;

use crate::{
    core::PubNubError,
    lib::alloc::{boxed::Box, vec, vec::Vec},
};

use cryptors::aes_gcm::AesGcmCryptor;

/// [`CryptoModule`] module extension with convenience methods.
impl CryptoModule {
    /// AES-CBC cryptor based module.
    ///
    /// Data _encryption_ and _decryption_ will be done by default using the
    /// [`AesCbcCryptor`]. For data _decryption_, the [`LegacyCryptor`] and
    /// AES-256-GCM (`ACGM`) are also registered. AES-256-GCM encryption is not
    /// available.
    ///
    /// Returns error if `cipher_key` is empty.
    pub fn new_aes_cbc_module<K>(cipher_key: K, use_random_iv: bool) -> Result<Self, PubNubError>
    where
        K: Into<Vec<u8>>,
    {
        let cipher_key = cipher_key.into();

        if cipher_key.is_empty() {
            return Err(PubNubError::CryptoInitialization {
                details: "Cipher key is empty".into(),
            });
        }

        Ok(Self::new(
            Box::new(AesCbcCryptor::new(cipher_key.clone())?),
            Some(vec![
                Box::new(LegacyCryptor::new(cipher_key.clone(), use_random_iv)?),
                Box::new(AesGcmCryptor::new(cipher_key)?),
            ]),
        ))
    }

    /// Legacy AES-CBC cryptor based module.
    ///
    /// Data _encryption_ and _decryption_ will be done by default using the
    /// [`LegacyCryptor`]. For data _decryption_, the [`AesCbcCryptor`] and
    /// AES-256-GCM (`ACGM`) are also registered so older and newer payloads can
    /// be read. AES-256-GCM encryption is not available.
    ///
    /// Returns error if `cipher_key` is empty.
    pub fn new_legacy_module<K>(cipher_key: K, use_random_iv: bool) -> Result<Self, PubNubError>
    where
        K: Into<Vec<u8>>,
    {
        let cipher_key = cipher_key.into();

        if cipher_key.is_empty() {
            return Err(PubNubError::CryptoInitialization {
                details: "Cipher key is empty".into(),
            });
        }

        Ok(Self::new(
            Box::new(LegacyCryptor::new(cipher_key.clone(), use_random_iv)?),
            Some(vec![
                Box::new(AesCbcCryptor::new(cipher_key.clone())?),
                Box::new(AesGcmCryptor::new(cipher_key)?),
            ]),
        ))
    }
}

#[cfg(test)]
mod it_should {
    use super::*;

    #[test]
    fn not_create_legacy_module_with_empty_cipher_key() {
        let crypto_module = CryptoModule::new_legacy_module("", false);
        assert!(crypto_module.is_err());
    }

    #[test]
    fn not_create_aes_cbc_module_with_empty_cipher_key() {
        let crypto_module = CryptoModule::new_aes_cbc_module("", false);
        assert!(crypto_module.is_err());
    }
}
