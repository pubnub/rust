//! # Crypto provider module
//!
//! This module contains the [`CryptoProvider`] trait, which is used to
//! implement a module that can be used to configure [`PubNubClientInstance`] or
//! for manual data encryption and decryption.

use crate::{
    core::PubNubError,
    lib::{alloc::vec::Vec, core::fmt::Debug},
};

/// Crypto provider trait.
pub trait CryptoProvider: Debug + Send + Sync {
    /// Encrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::Encryption`] if provided data can't be
    /// _encrypted_ or underlying cryptor misconfigured.
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, PubNubError>;

    /// Decrypt provided data.
    ///
    /// # Errors
    /// Should return an [`PubNubError::Decryption`] if provided data can't be
    /// _decrypted_ or underlying cryptor misconfigured.
    fn decrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, PubNubError>;
}
