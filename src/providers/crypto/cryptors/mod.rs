//! Cryptors module
//!
//! The module provides [`Cryptor`] trait implementations:
//! * [`LegacyCryptor`]
//! * [`AesCbcCryptor`]
//!
//! Actual implementations can be used to configure [`CryptoProvider`]
//! implementations for standalone usage or as part of [`PubNubClientInstance`]
//! for automated data _encryption_ and _decryption_.

#[doc(inline)]
pub use legacy::LegacyCryptor;
pub mod legacy;

#[doc(inline)]
pub use aes_cbc::AesCbcCryptor;
pub mod aes_cbc;
