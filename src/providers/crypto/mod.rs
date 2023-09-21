//! Crypto module
//!
//! This module contains a [`CryptoModule`] which allows to handle encrypted
//! data in backward compatible way. [`AesCbcCryptor`] and [`LegacyCryptor`]
//! cryptors available for [`CryptoModule`] configuration for data _encryption_
//! and _decryption_.

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

/// [`CryptoModule`] module extension with convenience methods.
impl CryptoModule {
    /// AES-CBC cryptor based module.
    ///
    /// Data _encryption_ and _decryption_ will be done by default using the
    /// [`AesCbcCryptor`]. In addition to the [`AesCbcCryptor`] for data
    /// _decryption_, the [`LegacyCryptor`] will be registered for
    /// backward-compatibility.
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
            Some(vec![Box::new(LegacyCryptor::new(
                cipher_key,
                use_random_iv,
            )?)]),
        ))
    }

    /// Legacy AES-CBC cryptor based module.
    ///
    /// Data _encryption_ and _decryption_ will be done by default using the
    /// [`LegacyCryptor`]. In addition to the [`LegacyCryptor`] for data
    /// _decryption_, the [`AesCbcCryptor`] will be registered for
    /// future-compatibility (which will help with gradual application updates).
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
            Some(vec![Box::new(AesCbcCryptor::new(cipher_key)?)]),
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
