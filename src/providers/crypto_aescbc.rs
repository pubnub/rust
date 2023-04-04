//! # AES-CBC Crypto Implementation
//!
//! This module contains [`AesCbcCrypto`] and [`AesCbcIv`] types.
//! It is used to encrypt and decrypt data sent and received from [`PubNub API`]
//! using the [`aes`] and [`cbc`] crates.
//!
//! It requires the [`aescbc` feature] to be enabled.
//!
//! [`PubNub API`]: https://www.pubnub.com/docs
//! [`aes`]: https://crates.io/crates/aes
//! [`cbc`]: https://crates.io/crates/cbc
//! [`aescbc` feature]: ../index.html#features
use crate::core::{error::PubNubError, Cryptor};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

/// AES-SHA256 encryptor type.
type Encryptor = cbc::Encryptor<aes::Aes256>;
/// AES-SHA256 encryptor type.
type Decryptor = cbc::Decryptor<aes::Aes256>;

/// AES cipher block size.
pub(crate) const AES_BLOCK_SIZE: usize = 16;

/// AES block cipher initialization vector.
///
/// It is intended to be used with [`AesCbcCrypto`].
///
/// [`AesCbcCrypto`]: ./struct.AesCbcCrypto.html
pub enum AesCbcIv {
    /// Hard-coded initialization vector.
    ///
    /// A value defined by the SDK will be used for each
    /// [`AesCbcCrypto.encrypt`] method call.
    ///
    /// This vector used for both encryption and decryption.
    ///
    /// [`AesCbcCrypto.encrypt`]: /struct.AesCbcCrypto.html#method.encrypt
    Constant,
    /// Random initialization vector.
    ///
    /// A new vector will be generated with each [`AesCbcCrypto.encrypt`] method
    /// call.
    ///
    /// This vector type used for encryption and decryption based on input data.
    ///
    /// [`AesCbcCrypto.encrypt`]: /struct.AesCbcCrypto.html#method.encrypt
    Random,
}

/// A crypto that uses the AES encryption algorithm with CBC mode.
///
/// Crypto provides interface fore data encryption and decryption.
///
/// # Examples
/// Create cryptor with cipher key and random initialization vector.
/// ```rust
/// # use pubnub::{
/// #     core::{error::PubNubError, Cryptor},
/// #     providers::crypto_aescbc::{AesCbcIv, AesCbcCrypto}
/// # };
/// #
/// # fn main() -> Result<(), PubNubError> {
/// let cryptor = AesCbcCrypto::new("enigma", AesCbcIv::Random)?;
/// # Ok(())
/// # }
/// ```
pub struct AesCbcCrypto {
    cipher_key: Vec<u8>,
    iv: Option<Vec<u8>>,
    iv_constant: bool,
}

impl AesCbcCrypto {
    /// Create an AES-CBC crypto for data encryption and decryption.
    ///
    /// It is **required** that the specified initialization vector be at
    /// maximum 16 bytes long. If you give us a value that is longer than 16
    /// bytes, it will be clipped or filled with zeros if it is shorter.
    ///
    /// # Errors
    /// Should return an [`PubNubError::CryptoInitializationError`] if cipher
    /// key or initialization vectors are empty.
    pub fn new<C>(cipher_key: C, iv: AesCbcIv) -> Result<Self, PubNubError>
    where
        C: Into<Vec<u8>>,
    {
        let iv_constant = matches!(iv, AesCbcIv::Constant);
        let cipher_key: Vec<u8> = cipher_key.into();

        if cipher_key.is_empty() {
            return Err(PubNubError::CryptoInitializationError(
                "Cipher key is empty".into(),
            ));
        }

        Ok(AesCbcCrypto {
            cipher_key: AesCbcCrypto::sha256_hex(cipher_key),
            iv: match iv {
                AesCbcIv::Constant => Some(b"0123456789012345".to_vec()),
                AesCbcIv::Random => None,
            },
            iv_constant,
        })
    }

    /// Calculate size of buffer for processed data.
    ///
    /// Buffers may require different sizes of allocated memory depending on
    /// operation and initialization vector type.
    fn estimated_buffer_size(&self, source: &[u8], encryption: bool) -> usize {
        let mut size = source.len();

        // More space required for encryption to align with AES cipher block
        // size.
        if encryption {
            size += (AES_BLOCK_SIZE - size % AES_BLOCK_SIZE) + AES_BLOCK_SIZE;
            // Reserve more space to store random initialization vector.
            if !self.iv_constant {
                size += AES_BLOCK_SIZE;
            }
        } else if !self.iv_constant {
            size -= AES_BLOCK_SIZE;
        }

        size
    }

    /// Data encryption initialization vector.
    ///
    /// Initialization vector which is suitable for current [`AesCbcCrypto`]
    /// configuration.
    ///
    /// [`AesCbcCrypto`]: ./struct.AesCbcCrypto.html
    fn encryption_iv(&self) -> Vec<u8> {
        match &self.iv {
            Some(iv) => Vec::from(iv.as_slice()),
            None => {
                let mut random = [0u8; AES_BLOCK_SIZE];
                thread_rng().try_fill(&mut random).ok();
                Vec::from(random)
            }
        }
    }

    /// Data decryption initialization vector.
    ///
    /// Initialization vector which is suitable for current [`AesCbcCrypto`]
    /// configuration.
    /// For [`AesCbcIv::Random`] variant initialization vector should be
    /// retrieved from received data.
    ///
    /// [`AesCbcCrypto`]: ./struct.AesCbcCrypto.html
    /// [`AesCbcIv::Random`]: ./enum.AesCbcIv.html#variant.Random
    fn decryption_iv(&self, source: &[u8]) -> Vec<u8> {
        match &self.iv {
            Some(iv) => iv.as_slice().to_vec(),
            None if source.len() > AES_BLOCK_SIZE => source[0..AES_BLOCK_SIZE].to_vec(),
            None => vec![],
        }
    }

    /// Calculate sha256 hash.
    /// The default crypto uses the [`sha2`] crate to calculate hash.
    ///
    /// [`shs2`]: https://crates.io/crates/sha2
    fn sha256_hex(data: Vec<u8>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data.as_slice());
        hasher
            .finalize()
            .iter()
            .take(AES_BLOCK_SIZE)
            .fold(String::new(), |acc, byte| format!("{}{:02x}", acc, byte))
            .into_bytes()
    }
}

impl Cryptor for AesCbcCrypto {
    /// Encrypt provided data.
    ///
    /// # Examples
    /// ```rust
    /// # use pubnub::{
    /// #     core::{error::PubNubError, Cryptor},
    /// #     providers::crypto_aescbc::{AesCbcIv, AesCbcCrypto}
    /// # };
    /// #
    /// # fn main() -> Result<(), PubNubError> {
    /// let cryptor = // AesCbcCrypto
    /// #    AesCbcCrypto::new("enigma", AesCbcIv::Random)?;
    /// let encrypted_data = cryptor.encrypt("Hello world!".as_bytes());
    /// match encrypted_data {
    ///     Ok(data) => println!("Encrypted data: {:?}", data),
    ///     Err(err) => eprintln!("Data encryption error: {}", err.to_string())
    /// };
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Should return an [`PubNubError::EncryptionError`] if provided data can't
    /// be encrypted or underlying cryptor misconfigured.
    fn encrypt<'en, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
    where
        T: Into<&'en [u8]>,
    {
        let iv = self.encryption_iv();
        let data = source.into();
        let mut buffer = vec![0u8; self.estimated_buffer_size(data, true)];
        let data_offset = if !self.iv_constant { AES_BLOCK_SIZE } else { 0 };
        let data_slice = &mut buffer[data_offset..];

        let result = Encryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .encrypt_padded_b2b_mut::<Pkcs7>(data, data_slice)
            .map_err(|err| PubNubError::EncryptionError(err.to_string()))?;
        let encrypted_len = result.len() + data_offset;

        // Prepend random initialization vector to encrypted data if required.
        if data_offset > 0 {
            buffer[0..data_offset].copy_from_slice(iv.as_slice());
        }

        // Adjust size of buffer to actual processed data length.
        buffer.resize(encrypted_len, 0);

        Ok(buffer)
    }

    /// Decrypt provided data.
    ///
    /// # Examples
    /// ```rust
    /// # use pubnub::{
    /// #     core::{error::PubNubError, Cryptor},
    /// #     providers::crypto_aescbc::{AesCbcIv, AesCbcCrypto}
    /// # };
    /// # use base64::{engine::general_purpose, Engine as _};
    /// #
    /// # fn main() -> Result<(), PubNubError> {
    /// let cryptor = // AesCbcCrypto
    /// #    AesCbcCrypto::new("enigma", AesCbcIv::Random)?;
    /// # let data_for_decryption = general_purpose::STANDARD
    /// #     .decode("fRm/rMArHgQuIuhuJMbXV8JLOUqf5sP72lGC4EaW98nNhmJltQcmCol9XXWgeDJC")
    /// #     .expect("Valid base64 encoded string required.");
    /// let encrypted_data = // &[u8]
    /// #    data_for_decryption.as_slice();
    /// let decrypted_data = cryptor.decrypt(encrypted_data);
    /// match decrypted_data {
    ///     Ok(data) => println!("Decrypted data: {:?}", String::from_utf8(data)), // "Hello there 🙃"
    ///     Err(err) => eprintln!("Data decryption error: {}", err.to_string())
    /// };
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Should return an [`PubNubError::DecryptionError`] if provided data can't
    /// be decrypted or underlying cryptor misconfigured.
    fn decrypt<'de, T>(&self, source: T) -> Result<Vec<u8>, PubNubError>
    where
        T: Into<&'de [u8]>,
    {
        let data = source.into();
        let iv = self.decryption_iv(data);
        let mut buffer = vec![0u8; self.estimated_buffer_size(data, false)];
        let data_offset = if !self.iv_constant { AES_BLOCK_SIZE } else { 0 };
        let data_slice = &data[data_offset..];

        let result = Decryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .decrypt_padded_b2b_mut::<Pkcs7>(data_slice, buffer.as_mut())
            .map_err(|err| PubNubError::DecryptionError(err.to_string()))?;

        // Adjust size of buffer to actual processed data length.
        let decrypted_len = result.len();
        buffer.resize(decrypted_len, 0);

        Ok(buffer)
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn create_cryptor_with_hardcoded_iv() {
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Constant).expect("Cryptor should be created");
        let iv = cryptor
            .iv
            .clone()
            .expect("Initialization vector should be created");
        assert_eq!(iv, "0123456789012345".as_bytes().to_vec());
        assert_eq!(cryptor.encryption_iv(), cryptor.encryption_iv());
    }

    #[test]
    fn create_cryptor_with_random_iv() {
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Random).expect("Cryptor should be created");
        assert!(cryptor.iv.is_none());
        assert_ne!(cryptor.encryption_iv(), cryptor.encryption_iv());
    }

    #[test]
    fn not_create_cryptor_with_empty_cipher_key() {
        let cryptor = AesCbcCrypto::new("", AesCbcIv::Random);
        assert!(cryptor.is_err());
    }

    #[test]
    fn encrypt_data_with_constant_iv() {
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Constant).expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        let encrypted2 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        assert_eq!(encrypted1, encrypted2);
        assert_ne!(
            "0123456789012345".as_bytes(),
            &encrypted1[0..AES_BLOCK_SIZE]
        );
        assert_eq!(
            general_purpose::STANDARD.encode(encrypted2),
            "4K7StI9dRz7utFsDHvuy082CQupbJvdwzrRja47qAV4="
        );
    }

    #[test]
    fn encrypt_data_with_random_iv() {
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Random).expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        let encrypted2 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        assert_ne!(encrypted1, encrypted2);
        assert_ne!(encrypted1[0..AES_BLOCK_SIZE], encrypted2[0..AES_BLOCK_SIZE]);
    }

    #[test]
    fn decrypt_data_with_constant_iv() {
        let encrypted = general_purpose::STANDARD
            .decode("4K7StI9dRz7utFsDHvuy082CQupbJvdwzrRja47qAV4=")
            .expect("Valid base64 encoded string required.");
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Constant).expect("Cryptor should be created");
        let decrypted = cryptor
            .decrypt(encrypted.as_slice())
            .expect("Data should be decrypted");
        assert_eq!(decrypted, "\"Hello there 🙃\"".as_bytes());
    }

    #[test]
    fn decrypt_data_with_random_iv() {
        let encrypted1 = general_purpose::STANDARD
            .decode("fRm/rMArHgQuIuhuJMbXV8JLOUqf5sP72lGC4EaW98nNhmJltQcmCol9XXWgeDJC")
            .expect("Valid base64 encoded string required.");
        let encrypted2 = general_purpose::STANDARD
            .decode("gk6glnaeb+8zeEvZR1q3sHyQV7xTo1pNf4cc4uJF+a2bK1fMY816Hc9I6j+gYR+5")
            .expect("Valid base64 encoded string required.");
        let cryptor =
            AesCbcCrypto::new("enigma", AesCbcIv::Random).expect("Cryptor should be created");
        let decrypted1 = cryptor
            .decrypt(encrypted1.as_slice())
            .expect("Data should be decrypted");
        let decrypted2 = cryptor
            .decrypt(encrypted2.as_slice())
            .expect("Data should be decrypted");
        assert_eq!(decrypted1, "\"Hello there 🙃\"".as_bytes());
        assert_eq!(decrypted1, decrypted2);
    }
}
