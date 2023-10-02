//! # AES-CBC cryptor module.
//!
//! Module contains [`AesCbcCryptor`] type which can be used for data encryption
//! and decryption.

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};

use crate::{
    core::{Cryptor, EncryptedData, PubNubError},
    lib::alloc::{format, string::ToString, vec, vec::Vec},
};

type Encryptor = cbc::Encryptor<aes::Aes256>;
/// AES-SHA256 encryptor type.
type Decryptor = cbc::Decryptor<aes::Aes256>;

/// Unique cryptor identifier
const IDENTIFIER: [u8; 4] = *b"ACRH";

/// AES cipher block size.
const AES_BLOCK_SIZE: usize = 16;

/// AES-CBC cryptor.
#[derive(Debug)]
pub struct AesCbcCryptor {
    /// Key for data _encryption_ / _decryption_.
    cipher_key: Vec<u8>,
}

impl AesCbcCryptor {
    /// Create AES-CBC cryptor.
    pub fn new<K>(cipher_key: K) -> Result<Self, PubNubError>
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
            cipher_key: Self::sha256(cipher_key),
        })
    }

    fn initialization_vector(&self) -> [u8; 16] {
        let mut random = [0u8; AES_BLOCK_SIZE];
        getrandom::getrandom(&mut random).ok();
        random
    }

    fn estimated_enc_buffer_size(&self, source: &[u8]) -> usize {
        // Adding padding which include additional AES cipher block size.
        source.len() + (AES_BLOCK_SIZE - source.len() % AES_BLOCK_SIZE) + AES_BLOCK_SIZE
    }

    fn estimated_dec_buffer_size(&self, source: &[u8]) -> usize {
        source.len()
    }

    fn sha256(data: Vec<u8>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data.as_slice());
        hasher.finalize().to_vec()
    }
}

impl Cryptor for AesCbcCryptor {
    fn identifier(&self) -> [u8; 4] {
        IDENTIFIER
    }

    fn encrypt(&self, data: Vec<u8>) -> Result<EncryptedData, PubNubError> {
        if data.is_empty() {
            return Err(PubNubError::Encryption {
                details: "Encrypted data is empty".into(),
            });
        }

        let mut buffer = vec![0u8; self.estimated_enc_buffer_size(&data)];
        let iv = self.initialization_vector();

        let result = Encryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .encrypt_padded_b2b_mut::<Pkcs7>(&data, &mut buffer)
            .map_err(|err| PubNubError::Encryption {
                details: err.to_string(),
            })?;
        let encrypted_len = result.len();

        // Adjust size of buffer to actual processed data length.
        buffer.resize(encrypted_len, 0);

        Ok(EncryptedData {
            metadata: Some(iv.to_vec()),
            data: buffer,
        })
    }

    fn decrypt(&self, data: EncryptedData) -> Result<Vec<u8>, PubNubError> {
        let mut buffer = vec![0u8; self.estimated_dec_buffer_size(&data.data)];
        let Some(iv) = data.metadata else {
            return Err(PubNubError::Decryption {
                details: "Initialization vector is missing from payload".into(),
            });
        };

        if iv.len().ne(&AES_BLOCK_SIZE) {
            return Err(PubNubError::Decryption {
                details: format!(
                    "Unexpected initialization vector size: {} bytes ({} bytes is expected)",
                    iv.len(),
                    AES_BLOCK_SIZE
                ),
            });
        }

        let result = Decryptor::new(self.cipher_key.as_slice().into(), iv.as_slice().into())
            .decrypt_padded_b2b_mut::<Pkcs7>(&data.data, buffer.as_mut())
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
    fn create_cryptor() {
        let cryptor = AesCbcCryptor::new("enigma").expect("Cryptor should be created");
        assert_ne!(
            cryptor.initialization_vector(),
            cryptor.initialization_vector()
        );
    }

    #[test]
    fn not_create_cryptor_with_empty_cipher_key() {
        let cryptor = AesCbcCryptor::new("");
        assert!(cryptor.is_err());
    }

    #[test]
    fn encrypt_data() {
        // Prefix includes potential header size.
        let prefix_size = AES_BLOCK_SIZE + 10;
        let cryptor = AesCbcCryptor::new("enigma").expect("Cryptor should be created");
        let encrypted1 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        let encrypted2 = cryptor
            .encrypt(Vec::from("\"Hello there 🙃\""))
            .expect("Data should be encrypted")
            .data;
        assert_ne!(encrypted1, encrypted2);
        assert_ne!(encrypted1[0..prefix_size], encrypted2[0..prefix_size]);
    }

    #[test]
    fn decrypt_data() {
        let header_offset = 10;
        let encrypted1 = general_purpose::STANDARD
            .decode(
                "UE5FRAFBQ1JIELzZwCmyT4vQLcjIAf8hSX2/mRRRby+egFPTqmwSKIFcjI1V/ig/y3M1iTlwknrTSw==",
            )
            .expect("Valid base64 encoded string required.");
        let encrypted2 = general_purpose::STANDARD
            .decode(
                "UE5FRAFBQ1JIECL8XmJWRSElf8c7ykQfMcnLQVc+Mta7ln3jcF7bHNmCk4nKMoyhPN19oMO5uVPxSA==",
            )
            .expect("Valid base64 encoded string required.");
        let cryptor = AesCbcCryptor::new("enigma").expect("Cryptor should be created");
        let decrypted1 = cryptor
            .decrypt(EncryptedData {
                metadata: Some(
                    encrypted1[header_offset..(header_offset + AES_BLOCK_SIZE)].to_vec(),
                ),
                data: encrypted1[(header_offset + AES_BLOCK_SIZE)..].to_vec(),
            })
            .expect("Data should be decrypted 1");
        let decrypted2 = cryptor
            .decrypt(EncryptedData {
                metadata: Some(
                    encrypted2[header_offset..(header_offset + AES_BLOCK_SIZE)].to_vec(),
                ),
                data: encrypted2[(header_offset + AES_BLOCK_SIZE)..].to_vec(),
            })
            .expect("Data should be decrypted 2");
        assert_eq!(decrypted1, "\"Hello there 🙃\"".as_bytes());
        assert_eq!(decrypted1, decrypted2);
    }
}
