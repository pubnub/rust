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
    /// Custom initialization vector.
    ///
    /// `Constant` user-provided vector which is used with each
    /// [`AesCbcCrypto.encrypt`] and [`AesCbcCrypto.decrypt`] method call method
    /// calls.
    ///
    /// # Examples
    /// Create an initialization vector using a short array. The resulting
    /// initialization vector will be aligned with the AES cipher block size
    /// (16 bytes) by filling rest with zeroes.
    /// ```rust
    /// # use pubnub::providers::crypto_aescbc::AesCbcIv;
    /// let iv = AesCbcIv::Custom(Vec::from("nonce"));
    /// ```
    ///
    /// [`AesCbcCrypto.encrypt`]: /struct.AesCbcCrypto.html#method.encrypt
    /// [`AesCbcCrypto.decrypt`]: /struct.AesCbcCrypto.html#method.decrypt
    Custom(Vec<u8>),
}

impl From<[u8; 16]> for AesCbcIv {
    /// Construct [`AesCbcIv::Custom`] from `[u8; 16]`.
    ///
    /// # Examples
    /// ```rust
    /// # use pubnub::providers::crypto_aescbc::AesCbcIv;
    ///  let iv: AesCbcIv = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5].into();
    /// # assert!(matches!(iv, AesCbcIv::Custom(_)));
    /// ```
    fn from(value: [u8; 16]) -> Self {
        Vec::from(value).into()
    }
}

impl From<&Vec<u8>> for AesCbcIv {
    /// Construct [`AesCbcIv::Custom`] from `&Vec<u8>`.
    ///
    /// It is **required** that the specified initialization vector be at
    /// maximum 16 bytes long. If you give us a value that is longer than 16
    /// bytes, it will be clipped or filled with zeros if it is shorter.
    fn from(value: &Vec<u8>) -> Self {
        Vec::from(value.as_slice()).into()
    }
}

impl From<Vec<u8>> for AesCbcIv {
    /// Construct [`AesCbcIv::Custom`] from `Vec<u8>`.
    ///
    /// # Examples
    /// Create an initialization vector using a short `Vec`. The resulting
    /// initialization vector will be aligned with the AES cipher block size
    /// (16 bytes) by filling rest with zeroes.
    /// ```rust
    /// # use pubnub::providers::crypto_aescbc::AesCbcIv;
    ///  let iv: AesCbcIv = Vec::from([0, 1, 2, 3, 4, 5]).into();
    /// # assert!(matches!(iv, AesCbcIv::Custom(_)));
    /// # assert_eq!(match &iv { AesCbcIv::Custom(vec) => vec.len(), _ => 0 }, 16);
    /// ```
    ///
    /// It is **required** that the specified initialization vector be at
    /// maximum 16 bytes long. If you give us a value that is longer than 16
    /// bytes, it will be clipped or filled with zeros if it is shorter.
    fn from(value: Vec<u8>) -> Self {
        if value.len() == AES_BLOCK_SIZE {
            return Self::Custom(value);
        } else if value.is_empty() {
            return Self::Custom(vec![]);
        }

        let length = value.len().min(AES_BLOCK_SIZE);
        let mut vec = [0u8; AES_BLOCK_SIZE];
        vec[0..length].copy_from_slice(&value[0..length]);
        Self::Custom(Vec::from(vec))
    }
}

impl From<&str> for AesCbcIv {
    /// Construct [`AesCbcIv::Custom`] from `&str`.
    ///
    /// # Examples
    /// Create an initialization vector using a too long string slice. The
    /// resulting initialization vector will be aligned with the AES cipher
    /// block size (16 bytes) by clipping extra bytes.
    /// ```rust
    /// # use pubnub::providers::crypto_aescbc::AesCbcIv;
    ///  let iv: AesCbcIv = "01234567890123456789".into();
    /// # assert!(matches!(iv, AesCbcIv::Custom(_)));
    /// # assert_eq!(match &iv { AesCbcIv::Custom(vec) => vec.len(), _ => 0 }, 16);
    /// ```
    ///
    /// It is **required** that the specified initialization vector be at
    /// maximum 16 bytes long. If you give us a value that is longer than 16
    /// bytes, it will be clipped or filled with zeros if it is shorter.
    fn from(value: &str) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl From<String> for AesCbcIv {
    /// Construct [`AesCbcIv::Custom`] from `String`.
    ///
    /// # Examples
    /// Create an initialization vector using a short `String`. The resulting
    /// initialization vector will be aligned with the AES cipher block size
    /// (16 bytes) by filling rest with zeroes.
    /// ```rust
    /// # use pubnub::providers::crypto_aescbc::AesCbcIv;
    ///  let iv: AesCbcIv = String::from("test-iv").into();
    /// # assert!(matches!(iv, AesCbcIv::Custom(_)));
    /// # assert_eq!(match &iv { AesCbcIv::Custom(vec) => vec.len(), _ => 0 }, 16);
    /// ```
    ///
    /// It is **required** that the specified initialization vector be at
    /// maximum 16 bytes long. If you give us a value that is longer than 16
    /// bytes, it will be clipped or filled with zeros if it is shorter.
    ///
    /// [`AesCbcIv::Custom`]: ./enum.AesCbcIv.html#variant.Custom
    fn from(value: String) -> Self {
        value.as_str().into()
    }
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
    pub fn new<C, I>(cipher_key: C, iv: I) -> Result<Self, PubNubError>
    where
        C: Into<Vec<u8>>,
        I: Into<AesCbcIv>,
    {
        let cipher_key: Vec<u8> = cipher_key.into();
        let vector: AesCbcIv = iv.into();
        let iv_constant = matches!(vector, AesCbcIv::Constant);

        if let AesCbcIv::Custom(custom_iv) = &vector {
            if custom_iv.is_empty() {
                return Err(PubNubError::CryptoInitializationError(
                    "Initialization vector is empty".into(),
                ));
            }
        }

        if cipher_key.is_empty() {
            return Err(PubNubError::CryptoInitializationError(
                "Cipher key is empty".into(),
            ));
        }

        Ok(AesCbcCrypto {
            cipher_key: AesCbcCrypto::sha256_hex(cipher_key),
            iv: match vector {
                AesCbcIv::Constant => Some(b"0123456789012345".to_vec()),
                AesCbcIv::Custom(custom) => Some(custom),
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
    /// For [`AesCbcIv::Random`] and [`AesCbcIv::Custom`] variants
    /// initialization vector should be retrieved from received data.
    ///
    /// [`AesCbcCrypto`]: ./struct.AesCbcCrypto.html
    /// [`AesCbcIv::Random`]: ./enum.AesCbcIv.html#variant.Random
    /// [`AesCbcIv::Custom`]: ./enum.AesCbcIv.html#variant.Custom
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
    fn encrypt<'en, T: Into<&'en [u8]>>(&self, source: T) -> Result<Vec<u8>, PubNubError> {
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
    fn decrypt<'de, T: Into<&'de [u8]>>(&self, source: T) -> Result<Vec<u8>, PubNubError> {
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
    use aes::cipher::typenum::assert_type_eq;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn create_custom_iv_from_array() {
        let iv_slice = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let iv: AesCbcIv = iv_slice.into();
        assert!(matches!(iv, AesCbcIv::Custom(_)));
        if let AesCbcIv::Custom(vec) = iv {
            assert_eq!(vec, iv_slice.to_vec())
        };
    }

    #[test]
    fn create_custom_iv_from_vector() {
        let iv_vec = &vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let iv: AesCbcIv = iv_vec.into();
        assert!(matches!(iv, AesCbcIv::Custom(_)));
        if let AesCbcIv::Custom(vec) = iv {
            assert_eq!(&vec, iv_vec);
        };
    }

    #[test]
    fn create_custom_iv_from_string_slice() {
        let iv_slice = "012345678901234567890";
        let iv: AesCbcIv = iv_slice.into();
        assert!(matches!(iv, AesCbcIv::Custom(_)));
        if let AesCbcIv::Custom(vec) = iv {
            assert_eq!(vec.len(), AES_BLOCK_SIZE);
            assert_eq!(vec, iv_slice[0..AES_BLOCK_SIZE].as_bytes().to_vec());
        };
    }

    #[test]
    fn create_custom_iv_from_string() {
        let iv_vec = "01234567890".to_string();
        let iv: AesCbcIv = iv_vec.clone().into();
        assert!(matches!(iv, AesCbcIv::Custom(_)));
        if let AesCbcIv::Custom(vec) = iv {
            assert_eq!(vec.len(), AES_BLOCK_SIZE);
            assert_eq!(&vec[0.."01234567890".len()], iv_vec.as_bytes());
            // When to short vector provided, rest of it filled with zeroes.
            assert_eq!(
                &vec[iv_vec.len()..AES_BLOCK_SIZE],
                &[0u8; AES_BLOCK_SIZE - "01234567890".len()]
            );
        };
    }

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
    fn create_cryptor_with_custom_iv() {
        let cryptor = AesCbcCrypto::new("enigma", AesCbcIv::Custom("nonce".into()))
            .expect("Cryptor should be created");
        let iv = cryptor
            .iv
            .clone()
            .expect("Initialization vector should be created");
        assert_eq!(iv, "nonce".as_bytes().to_vec());
        assert_eq!(cryptor.encryption_iv(), cryptor.encryption_iv());
    }

    #[test]
    fn create_cryptor_with_custom_iv_from_string() {
        let cryptor = AesCbcCrypto::new("enigma", "nonce").expect("Cryptor should be created");
        cryptor
            .iv
            .clone()
            .expect("Initialization vector should be created");
        assert_eq!(cryptor.encryption_iv(), cryptor.encryption_iv());
    }

    #[test]
    fn not_create_cryptor_with_empty_cipher_key() {
        let cryptor = AesCbcCrypto::new("", "nonce");
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
            general_purpose::STANDARD.encode(encrypted2.clone()),
            "4K7StI9dRz7utFsDHvuy082CQupbJvdwzrRja47qAV4="
        );
    }

    #[test]
    fn encrypt_data_with_random_iv() {
        // "aM5AVD8eO0B5qbu+IY8TGBpbMi7tPs+iwNqa9HUYVOpGarTb/jNtj/jC7+5VVYNV"
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
        // Encrypted 1: fRm/rMArHgQuIuhuJMbXV8JLOUqf5sP72lGC4EaW98nNhmJltQcmCol9XXWgeDJC
        // Encrypted 2: gk6glnaeb+8zeEvZR1q3sHyQV7xTo1pNf4cc4uJF+a2bK1fMY816Hc9I6j+gYR+5
    }

    #[test]
    fn encrypt_data_with_custom_iv() {
        let cryptor = AesCbcCrypto::new("enigma", "test-iv").expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        let encrypted2 = cryptor
            .encrypt("\"Hello there 🙃\"".as_bytes())
            .expect("Data should be encrypted");
        assert_eq!(encrypted1, encrypted2);
        assert_eq!("test-iv".as_bytes(), &encrypted1[0.."test-iv".len()]);
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
