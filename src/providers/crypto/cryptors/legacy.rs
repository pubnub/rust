//! # Legacy AES-CBC cryptor module.
//!
//! Module contains [`LegacyCryptor`] type which can be used for data encryption
//! and decryption.

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};

use crate::{
    core::{Cryptor, EncryptedData, PubNubError},
    lib::alloc::{
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    },
};

type Encryptor = cbc::Encryptor<aes::Aes256>;
/// AES-SHA256 encryptor type.
type Decryptor = cbc::Decryptor<aes::Aes256>;

/// Unique cryptor identifier
const IDENTIFIER: [u8; 4] = [0x00u8; 4];

/// AES cipher block size.
const AES_BLOCK_SIZE: usize = 16;

/// Legacy cryptor.
///
/// Legacy AES-CBC cryptor which let use random or hardcoded initialization
/// vector and key with low entropy issue.
#[derive(Debug)]
pub struct LegacyCryptor {
    /// Whether random IV should be used.
    ///
    /// With enabled random IV it will become part of cryptor-defined fields.
    use_random_iv: bool,

    /// Key for data encryption / decryption
    cipher_key: Vec<u8>,
}

impl LegacyCryptor {
    /// Create Legacy AES-CBC cryptor.
    pub fn new<K>(cipher_key: K, use_random_iv: bool) -> Result<Self, PubNubError>
    where
        K: Into<Vec<u8>>,
    {
        let cipher_key = cipher_key.into();

        if cipher_key.is_empty() {
            return Err(PubNubError::CryptoInitialization {
                details: "Cipher key is empty".into(),
            });
        }

        Ok(Self {
            use_random_iv,
            cipher_key: Self::sha256_hex(cipher_key),
        })
    }

    fn initialization_vector(&self) -> [u8; 16] {
        if self.use_random_iv {
            let mut random = [0u8; AES_BLOCK_SIZE];
            getrandom::getrandom(&mut random).ok();
            random
        } else {
            *b"0123456789012345"
        }
    }

    fn estimated_enc_buffer_size(&self, source: &[u8]) -> usize {
        // Adding padding which include additional AES cipher block size.
        let padding = (AES_BLOCK_SIZE - source.len() % AES_BLOCK_SIZE) + AES_BLOCK_SIZE;
        if self.use_random_iv {
            // Reserve more space to store random initialization vector.
            source.len() + padding + AES_BLOCK_SIZE
        } else {
            source.len() + padding
        }
    }

    fn estimated_dec_buffer_size(&self, source: &[u8]) -> usize {
        // Subtract size of random initialization vector (if used).
        source.len()
            - if self.use_random_iv {
                AES_BLOCK_SIZE
            } else {
                0
            }
    }

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

impl Cryptor for LegacyCryptor {
    fn identifier(&self) -> [u8; 4] {
        IDENTIFIER
    }

    fn encrypt(&self, data: Vec<u8>) -> Result<EncryptedData, PubNubError> {
        let mut buffer = vec![0u8; self.estimated_enc_buffer_size(&data)];
        let data_offset = if self.use_random_iv {
            AES_BLOCK_SIZE
        } else {
            0
        };
        let data_slice = &mut buffer[data_offset..];
        let iv = self.initialization_vector();

        let result = Encryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .encrypt_padded_b2b_mut::<Pkcs7>(&data, data_slice)
            .map_err(|err| PubNubError::Encryption {
                details: err.to_string(),
            })?;
        let encrypted_len = result.len() + data_offset;

        // Prepend random initialization vector to encrypted data if required.
        if data_offset > 0 {
            buffer[0..data_offset].copy_from_slice(iv.as_slice());
        }

        // Adjust size of buffer to actual processed data length.
        buffer.resize(encrypted_len, 0);

        Ok(EncryptedData {
            metadata: None,
            data: buffer,
        })
    }

    fn decrypt(&self, data: EncryptedData) -> Result<Vec<u8>, PubNubError> {
        let mut buffer = vec![0u8; self.estimated_dec_buffer_size(&data.data)];
        let data_offset = if self.use_random_iv {
            AES_BLOCK_SIZE
        } else {
            0
        };
        let iv = if self.use_random_iv {
            data.data[0..AES_BLOCK_SIZE].to_vec()
        } else {
            self.initialization_vector().to_vec()
        };
        let data_slice = &data.data[data_offset..];

        let result = Decryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .decrypt_padded_b2b_mut::<Pkcs7>(data_slice, buffer.as_mut())
            .map_err(|err| PubNubError::Decryption {
                details: err.to_string(),
            })?;

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
        let cryptor = LegacyCryptor::new("enigma", false).expect("Cryptor should be created");
        let iv = cryptor.initialization_vector();
        assert_eq!(&iv, b"0123456789012345");
        assert_eq!(
            cryptor.initialization_vector(),
            cryptor.initialization_vector()
        );
    }

    #[test]
    fn create_cryptor_with_random_iv() {
        let cryptor = LegacyCryptor::new("enigma", true).expect("Cryptor should be created");
        assert_ne!(
            cryptor.initialization_vector(),
            cryptor.initialization_vector()
        );
    }

    #[test]
    fn not_create_cryptor_with_empty_cipher_key() {
        let cryptor = LegacyCryptor::new("", true);
        assert!(cryptor.is_err());
    }

    #[test]
    fn encrypt_data_with_constant_iv() {
        let cryptor = LegacyCryptor::new("enigma", false).expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        let encrypted2 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        assert_eq!(encrypted1, encrypted2);
        assert_ne!(b"0123456789012345", &encrypted1[0..AES_BLOCK_SIZE]);
        assert_eq!(
            general_purpose::STANDARD.encode(encrypted2),
            "4K7StI9dRz7utFsDHvuy082CQupbJvdwzrRja47qAV4="
        );
    }

    #[test]
    fn encrypt_data_with_random_iv() {
        let cryptor = LegacyCryptor::new("enigma", true).expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        let encrypted2 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        assert_ne!(encrypted1, encrypted2);
        assert_ne!(encrypted1[0..AES_BLOCK_SIZE], encrypted2[0..AES_BLOCK_SIZE]);
    }

    #[test]
    fn decrypt_data_with_constant_iv() {
        let encrypted = general_purpose::STANDARD
            .decode("4K7StI9dRz7utFsDHvuy082CQupbJvdwzrRja47qAV4=")
            .expect("Valid base64 encoded string required.");
        let cryptor = LegacyCryptor::new("enigma", false).expect("Cryptor should be created");
        let decrypted = cryptor
            .decrypt(EncryptedData {
                metadata: None,
                data: encrypted,
            })
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
        let cryptor = LegacyCryptor::new("enigma", true).expect("Cryptor should be created");
        let decrypted1 = cryptor
            .decrypt(EncryptedData {
                metadata: None,
                data: encrypted1,
            })
            .expect("Data should be decrypted");
        let decrypted2 = cryptor
            .decrypt(EncryptedData {
                metadata: None,
                data: encrypted2,
            })
            .expect("Data should be decrypted");
        assert_eq!(decrypted1, "\"Hello there 🙃\"".as_bytes());
        assert_eq!(decrypted1, decrypted2);
    }
}
