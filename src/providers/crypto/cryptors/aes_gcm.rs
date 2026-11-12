//! AES-256-GCM cryptor.
//!
//! Decrypts `ACGM` payloads. Encryption stays disabled until a later phase.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

use crate::{
    core::{Cryptor, EncryptedData, PubNubError},
    lib::alloc::vec::Vec,
};

/// Cryptor identifier written into the PNED header.
const IDENTIFIER: [u8; 4] = *b"ACGM";

/// AES-GCM nonce size. The nonce is the only cryptor metadata.
const GCM_NONCE_SIZE: usize = 12;

/// Authentication tag size. The tag is appended to the ciphertext.
const GCM_TAG_SIZE: usize = 16;

/// AES-256-GCM cryptor (`ACGM`).
///
/// Built-in crypto modules register this cryptor for decryption only.
#[derive(Debug)]
pub(crate) struct AesGcmCryptor {
    /// SHA-256 digest of the caller-supplied cipher key.
    cipher_key: Vec<u8>,
}

impl AesGcmCryptor {
    /// Create an AES-256-GCM cryptor.
    ///
    /// Returns an error if `cipher_key` is empty.
    pub(crate) fn new<K>(cipher_key: K) -> Result<Self, PubNubError>
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
            cipher_key: sha256(cipher_key),
        })
    }
}

impl Cryptor for AesGcmCryptor {
    fn identifier(&self) -> [u8; 4] {
        IDENTIFIER
    }

    fn encrypt(&self, _data: Vec<u8>) -> Result<EncryptedData, PubNubError> {
        Err(PubNubError::Encryption {
            details: "AES-GCM encryption is not available".into(),
        })
    }

    fn decrypt(&self, data: EncryptedData) -> Result<Vec<u8>, PubNubError> {
        let Some(nonce) = data.metadata else {
            return Err(decryption_error());
        };
        if nonce.len() != GCM_NONCE_SIZE || data.data.len() < GCM_TAG_SIZE {
            return Err(decryption_error());
        }

        let cipher = Aes256Gcm::new_from_slice(&self.cipher_key).map_err(|_| decryption_error())?;
        cipher
            .decrypt(Nonce::from_slice(&nonce), data.data.as_ref())
            .map_err(|_| decryption_error())
    }
}

fn sha256(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data.as_slice());
    hasher.finalize().to_vec()
}

fn decryption_error() -> PubNubError {
    PubNubError::Decryption {
        details: "decryption error".into(),
    }
}

#[cfg(test)]
mod it_should {
    use super::*;
    use crate::{core::CryptoProvider, providers::crypto::CryptoModule};

    struct KnownCiphertext {
        name: &'static str,
        cipher_key: &'static str,
        plaintext: &'static [u8],
        payload: &'static str,
    }

    const KNOWN_CIPHERTEXTS: &[KnownCiphertext] = &[
        KnownCiphertext {
            name: "hello",
            cipher_key: "enigma",
            plaintext: b"Hello world",
            payload: "504e4544014143474d0c0102030405060708090a0b0cea1b5780788b593294f898dabd1d02d7219c77218ad262c4a9aecc",
        },
        KnownCiphertext {
            name: "unicode",
            cipher_key: "enigma",
            plaintext: "héllo 你好".as_bytes(),
            payload: "504e4544014143474d0c111111111111111111111111d138fbfc4bd0c337e29d9f2c98a5dd072360f4433e2979f440fc7827b3",
        },
        KnownCiphertext {
            name: "binary",
            cipher_key: "myCipherKey",
            plaintext: b"\x00\xff\x7f\x80",
            payload: "504e4544014143474d0c000102030405060708090a0bc411f0f18fefca7becf7235b4d09b970d4d56d19",
        },
        KnownCiphertext {
            name: "json-emoji",
            cipher_key: "enigma",
            plaintext: r#"{"text":"Hello, 世界! 🌍","n":42}"#.as_bytes(),
            payload: "504e4544014143474d0cffffffffffffffffffffffff0d54b73ce2509d4e7655043f5b103cb4f1e78738a73a2f56109813955359b4784774de1954f835d4d739805631c5cdcc3467607259",
        },
        KnownCiphertext {
            name: "one-block",
            cipher_key: "enigma",
            plaintext: b"0123456789abcdef",
            payload: "504e4544014143474d0c8f5a2d1b6e9c4ef0a37d83b9fd24a488008bd084305aaee605b349028c21b92d48f4dd5c71e99dce881c2ec9",
        },
        KnownCiphertext {
            name: "multi-block-json",
            cipher_key: "enigma",
            plaintext: r#"{"text":"Hello, 世界! 🌍","n":42,"list":[1,2,3],"nested":{"ok":true}}"#.as_bytes(),
            payload: "504e4544014143474d0cffffffffffffffffffffffff0d54b73ce2509d4e7655043f5b103cb4f1e78738a73a2f56109813955359b4784774de19050b4f14f7a439a859ac6c327856fe1af874bbe888945c603004176ddab22578e407663346d307669e83fa4c9a8b13743a07a21b7f",
        },
        KnownCiphertext {
            name: "unicode-key-multiblock",
            cipher_key: "ключ 🔑 key",
            plaintext: "Line one\n\tLine two with \"quotes\" & symbols <>/\\ — spans several AES blocks.".as_bytes(),
            payload: "504e4544014143474d0c000000000000000000000000326052d9ff1b757ed8810f5d6f87a7c021920eb09b82e47049efecf5094f6f5837691e9fb66539dfc47d434cc01b783527c4d62ae2d8163043f86dc7ac405b90971ceaa07134add2be50dfd60f8390572d9b05255f3097614e888fcca4",
        },
        KnownCiphertext {
            name: "single-byte",
            cipher_key: "customKey",
            plaintext: b"a",
            payload: "504e4544014143474d0ca1a2a3a4a5a6a7a8a9aaabacfbeef1ccce8f5a09087348786b99b75076",
        },
    ];

    const HELLO_PAYLOAD: &str = KNOWN_CIPHERTEXTS[0].payload;

    #[test]
    fn not_create_cryptor_with_empty_cipher_key() {
        assert!(AesGcmCryptor::new("").is_err());
    }

    #[test]
    fn report_acgm_identifier() {
        let cryptor = AesGcmCryptor::new("enigma").expect("cryptor");
        assert_eq!(cryptor.identifier(), *b"ACGM");
    }

    #[test]
    fn decrypt_known_ciphertexts() {
        for case in KNOWN_CIPHERTEXTS {
            let payload = decode_hex(case.payload);
            let encrypted = split_gcm_payload(&payload);
            assert_eq!(
                encrypted.metadata.as_ref().map(Vec::len),
                Some(GCM_NONCE_SIZE),
                "{}",
                case.name
            );

            let cryptor = AesGcmCryptor::new(case.cipher_key).expect("cryptor");
            let decrypted = cryptor
                .decrypt(encrypted)
                .unwrap_or_else(|err| panic!("{}: {err}", case.name));
            assert_eq!(decrypted, case.plaintext, "{}", case.name);

            for module in both_modules(case.cipher_key) {
                let out = module
                    .decrypt(payload.clone())
                    .unwrap_or_else(|err| panic!("{}: {err}", case.name));
                assert_eq!(out, case.plaintext, "{}", case.name);
            }
        }
    }

    #[test]
    fn decrypt_empty_plaintext() {
        let nonce = decode_hex("090807060504030201000102");
        let ciphertext = seal(b"enigma", &nonce, b"");
        assert_eq!(ciphertext.len(), GCM_TAG_SIZE);

        let cryptor = AesGcmCryptor::new("enigma").expect("cryptor");
        let decrypted = cryptor
            .decrypt(EncryptedData {
                metadata: Some(nonce.clone()),
                data: ciphertext.clone(),
            })
            .expect("decrypt");
        assert!(decrypted.is_empty());

        let payload = pned_payload(&nonce, &ciphertext);
        for module in both_modules("enigma") {
            let out = module.decrypt(payload.clone()).expect("module decrypt");
            assert!(out.is_empty());
        }
    }

    #[test]
    fn reject_invalid_payloads() {
        let payload = decode_hex(HELLO_PAYLOAD);
        let encrypted = split_gcm_payload(&payload);
        let nonce = encrypted.metadata.clone().expect("nonce");
        let ciphertext = encrypted.data.clone();
        let cryptor = AesGcmCryptor::new("enigma").expect("cryptor");

        let mut tampered = ciphertext.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0xff;
        let err = cryptor
            .decrypt(EncryptedData {
                metadata: Some(nonce.clone()),
                data: tampered,
            })
            .expect_err("tampered tag");
        assert_decryption_error(&err);

        let mut tampered_payload = payload.clone();
        let last = tampered_payload.len() - 1;
        tampered_payload[last] ^= 0xff;
        let err = both_modules("enigma")[0]
            .decrypt(tampered_payload)
            .expect_err("module tampered tag");
        assert_decryption_error(&err);

        let err = AesGcmCryptor::new("other-key")
            .expect("cryptor")
            .decrypt(EncryptedData {
                metadata: Some(nonce.clone()),
                data: ciphertext.clone(),
            })
            .expect_err("wrong key");
        assert_decryption_error(&err);

        let err = cryptor
            .decrypt(EncryptedData {
                metadata: Some(nonce[..GCM_NONCE_SIZE - 1].to_vec()),
                data: ciphertext.clone(),
            })
            .expect_err("short nonce");
        assert_decryption_error(&err);

        let err = cryptor
            .decrypt(EncryptedData {
                metadata: Some(nonce),
                data: ciphertext[..GCM_TAG_SIZE - 1].to_vec(),
            })
            .expect_err("short body");
        assert_decryption_error(&err);

        let err = cryptor
            .decrypt(EncryptedData {
                metadata: None,
                data: ciphertext,
            })
            .expect_err("missing nonce");
        assert_decryption_error(&err);
    }

    #[test]
    fn encrypt_is_disabled() {
        let cryptor = AesGcmCryptor::new("enigma").expect("cryptor");
        for data in [Vec::new(), b"Hello world".to_vec()] {
            let err = cryptor.encrypt(data).expect_err("encrypt");
            assert_eq!(
                err,
                PubNubError::Encryption {
                    details: "AES-GCM encryption is not available".into(),
                }
            );
        }
    }

    #[test]
    fn factories_keep_existing_encryptors() {
        let plaintext = b"Hello world".to_vec();
        let payload = decode_hex(HELLO_PAYLOAD);

        let cbc = CryptoModule::new_aes_cbc_module("enigma", true).expect("cbc module");
        let legacy = CryptoModule::new_legacy_module("enigma", false).expect("legacy module");

        assert_eq!(
            cbc.decrypt(payload.clone()).expect("cbc decrypt"),
            plaintext
        );
        assert_eq!(legacy.decrypt(payload).expect("legacy decrypt"), plaintext);

        let cbc_encrypted = cbc.encrypt(plaintext.clone()).expect("cbc encrypt");
        assert!(cbc_encrypted.starts_with(b"PNED\x01ACRH"));
        assert_eq!(
            cbc.decrypt(cbc_encrypted).expect("cbc round trip"),
            plaintext
        );

        let legacy_encrypted = legacy.encrypt(plaintext.clone()).expect("legacy encrypt");
        assert!(!legacy_encrypted.starts_with(b"PNED"));
        assert_eq!(
            legacy.decrypt(legacy_encrypted).expect("legacy round trip"),
            plaintext
        );
    }

    fn both_modules(cipher_key: &str) -> [CryptoModule; 2] {
        [
            CryptoModule::new_aes_cbc_module(cipher_key, true).expect("cbc module"),
            CryptoModule::new_legacy_module(cipher_key, true).expect("legacy module"),
        ]
    }

    fn split_gcm_payload(payload: &[u8]) -> EncryptedData {
        const PREFIX: usize = 10;
        EncryptedData {
            metadata: Some(payload[PREFIX..PREFIX + GCM_NONCE_SIZE].to_vec()),
            data: payload[PREFIX + GCM_NONCE_SIZE..].to_vec(),
        }
    }

    fn pned_payload(nonce: &[u8], body: &[u8]) -> Vec<u8> {
        let mut payload = Vec::with_capacity(10 + nonce.len() + body.len());
        payload.extend_from_slice(b"PNED\x01ACGM");
        payload.push(nonce.len() as u8);
        payload.extend_from_slice(nonce);
        payload.extend_from_slice(body);
        payload
    }

    fn seal(cipher_key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new_from_slice(&sha256(cipher_key.to_vec())).expect("key");
        cipher
            .encrypt(Nonce::from_slice(nonce), plaintext)
            .expect("seal")
    }

    fn decode_hex(value: &str) -> Vec<u8> {
        assert!(value.len() % 2 == 0, "odd hex length");
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).expect("hex digit"))
            .collect()
    }

    fn assert_decryption_error(err: &PubNubError) {
        assert_eq!(
            err,
            &PubNubError::Decryption {
                details: "decryption error".into(),
            }
        );
        assert!(!err.to_string().contains("Hello world"));
    }
}
