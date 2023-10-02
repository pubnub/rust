//! # Crypto module

use crate::{
    core::{CryptoProvider, Cryptor, EncryptedData, PubNubError},
    lib::alloc::{boxed::Box, format, string::String, vec, vec::Vec},
    providers::crypto::CryptorHeader,
};

/// PubNub client crypto module.
///
/// Module used by [`PubNubClientInstance`] for automated data encryption and
/// decryption. This module can be used separately for manual data encryption
/// and decryption.
///
/// # Examples
/// Create crypto module using convenience functions:
///
/// * with currently used implementation for encryption:
/// ```rust
/// # use pubnub::{
/// #     core::PubNubError,
/// #     providers::crypto::CryptoModule
/// # };
/// #
/// # fn main() -> Result<(), PubNubError> {
/// let crypto_module = CryptoModule::new_legacy_module("enigma", true)?;
/// # Ok(())
/// # }
/// ```
///
/// * with newer cryptor version:
/// ```rust
/// # use pubnub::{
/// #     core::PubNubError,
/// #     providers::crypto::CryptoModule
/// # };
/// #
/// # fn main() -> Result<(), PubNubError> {
/// let crypto_module = CryptoModule::new_aes_cbc_module("enigma", true)?;
/// # Ok(())
/// # }
/// ```
///
/// Create cryptor module with custom set of cryptors:
/// ```rust
/// # use pubnub::{
/// #     core::PubNubError,
/// #     providers::crypto::{AesCbcCryptor, CryptoModule, LegacyCryptor}
/// # };
/// #
/// # fn main() -> Result<(), PubNubError> {
/// let crypto_module = CryptoModule::new(
///     Box::new(LegacyCryptor::new("enigma".clone(), true)?),
///     Some(vec![Box::new(AesCbcCryptor::new("enigma")?)]),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CryptoModule {
    /// Default cryptor.
    ///
    /// Default cryptor used for data _encryption_ and _decryption_.
    default: Box<dyn Cryptor>,

    /// List of known cryptors.
    ///
    /// List of cryptors which is used to _decrypt_ data encrypted by previously
    /// used cryptors.
    cryptors: Option<Vec<Box<dyn Cryptor>>>,
}

impl CryptoModule {
    /// Create crypto module.
    ///
    /// `default` used to _encrypt_ and _decrypt_ corresponding data while rest
    /// of `cryptors` will be used to _decrypt_ data encrypted by previously
    /// used cryptors.
    pub fn new(default: Box<dyn Cryptor>, cryptors: Option<Vec<Box<dyn Cryptor>>>) -> Self {
        Self { default, cryptors }
    }

    /// Find cryptor suitable to handle data.
    fn cryptor_with_identifier(&self, header: &CryptorHeader) -> Option<&dyn Cryptor> {
        // Check whether there is no header - it mean that legacy cryptor should
        // be used.
        let identifier = header.identifier().unwrap_or([0x00u8; 4]);

        // Check whether default cryptor can handle data or not.
        if self.default.identifier().eq(&identifier) {
            return Some(self.default.as_ref());
        }

        // Return first matching cryptor.
        self.cryptors.as_ref().and_then(|cryptors| {
            cryptors
                .iter()
                .position(|cryptor| cryptor.identifier().eq(&identifier))
                .map(|position| cryptors[position].as_ref())
        })
    }
}

impl CryptoProvider for CryptoModule {
    /// Encrypt provided data.
    ///
    /// # Examples
    /// ```rust
    /// # use pubnub::{
    /// #     core::{PubNubError, CryptoProvider},
    /// #     providers::crypto::CryptoModule
    /// # };
    /// #
    /// # fn main() -> Result<(), PubNubError> {
    /// let crypto_module = CryptoModule::new_aes_cbc_module("enigma", true)?;
    /// let result = crypto_module.encrypt(Vec::from("hello world!"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, PubNubError> {
        // Encrypting provided data.
        let encrypted = self.default.encrypt(data)?;

        // Compute cryptor header.
        let header = CryptorHeader::new(self.default.identifier(), &encrypted.metadata);

        // Concatenate encrypted data with header into single payload.
        let mut payload = vec![0; header.len()];
        let mut pos = header.len();
        let header_data: Vec<u8> = header.into();
        payload.splice(0..pos, header_data);
        if let Some(metadata) = encrypted.metadata {
            pos -= metadata.len();
            payload.splice(pos..(pos + metadata.len()), metadata.clone());
        }
        payload.extend(encrypted.data);

        Ok(payload)
    }

    /// Decrypt provided data.
    ///
    /// # Examples
    /// ```rust
    /// # use base64::{engine::general_purpose, Engine as _};
    /// # use pubnub::{
    /// #     core::{PubNubError, CryptoProvider},
    /// #     providers::crypto::CryptoModule
    /// # };
    /// #
    /// # fn main() -> Result<(), PubNubError> {
    /// let encrypted_data = //
    /// #     general_purpose::STANDARD
    /// #         .decode(
    /// #             "UE5FRAFBQ1JIELzZwCmyT4vQLcjIAf8hSX2/mRRRby+egFPTqmwSKIFcjI1V/ig/y3M1iTlwknrTSw==",
    /// #         )
    /// #         .expect("Valid base64 encoded string required.");
    /// let crypto_module = CryptoModule::new_aes_cbc_module("enigma", true)?;
    /// let result = crypto_module.decrypt(encrypted_data)?;
    /// #
    /// # assert_eq!(result, "\"Hello there 🙃\"".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    fn decrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, PubNubError> {
        if data.is_empty() {
            return Err(PubNubError::Decryption {
                details: "Decrypted data is empty".into(),
            });
        }

        // Try read header content from received data.
        let header = CryptorHeader::try_from(&data)?;

        // Checking whether any cryptor for specified identifier has been found.
        let Some(cryptor) = self.cryptor_with_identifier(&header) else {
            let identifier = header.identifier().unwrap_or(*b"UNKN");
            // Looks like payload with unknown cryptor identifier has been received.
            Err(PubNubError::UnknownCryptor {
                details: format!(
                    "Decrypting data created by unknown cryptor. Please make sure to register {} \
                    or update SDK",
                    String::from_utf8(identifier.to_vec()).unwrap_or("non-utf8 identifier".into())
                ),
            })?
        };

        let metadata = match header.data_size() {
            Some(size) => {
                let offset = header.len() - size;
                Some(data[offset..(offset + size)].to_vec())
            }
            None => None,
        };

        cryptor.decrypt(EncryptedData {
            metadata,
            data: data[header.len()..].to_vec(),
        })
    }
}

#[cfg(test)]
mod it_should {
    use super::*;

    const IDENTIFIER: [u8; 4] = *b"ABCD";
    const METADATA: [u8; 17] = *b"this-is-meta-data";
    const ENCRYPTED_DATA: [u8; 22] = *b"this-is-encrypted-data";
    const DECRYPTED_DATA: [u8; 22] = *b"this-is-decrypted-data";

    #[derive(Debug)]
    struct MyCryptor;

    impl Cryptor for MyCryptor {
        fn identifier(&self) -> [u8; 4] {
            IDENTIFIER
        }

        fn encrypt(&self, data: Vec<u8>) -> Result<EncryptedData, PubNubError> {
            assert_eq!(data, DECRYPTED_DATA.to_vec());
            Ok(EncryptedData {
                metadata: Some(METADATA.to_vec()),
                data: ENCRYPTED_DATA.to_vec(),
            })
        }

        fn decrypt(&self, data: EncryptedData) -> Result<Vec<u8>, PubNubError> {
            assert_eq!(data.metadata.unwrap(), METADATA.to_vec());
            assert_eq!(data.data, ENCRYPTED_DATA.to_vec());

            Ok(DECRYPTED_DATA.to_vec())
        }
    }

    #[test]
    fn add_crypto_header_v1_data() {
        let cryptor_module = CryptoModule::new(Box::new(MyCryptor), None);
        let encrypt_result = cryptor_module.encrypt(DECRYPTED_DATA.to_vec());

        let Ok(data) = encrypt_result else {
            panic!("Encryption should be successful")
        };

        assert_eq!(data[0..4], *b"PNED");
        assert_eq!(data[4], 1);
        assert_eq!(data[5..9], IDENTIFIER);
        assert_eq!(data[9] as usize, METADATA.len());
        assert_eq!(data[10..(10 + data[9] as usize)], METADATA)
    }

    #[test]
    fn encrypt_data() {
        let cryptor_module = CryptoModule::new(Box::new(MyCryptor), None);
        let encrypt_result = cryptor_module.encrypt(DECRYPTED_DATA.to_vec());

        let Ok(data) = encrypt_result else {
            panic!("Encryption should be successful")
        };

        assert_eq!(data[(10 + data[9] as usize)..], ENCRYPTED_DATA)
    }

    #[test]
    fn decrypt_data() {
        let mut encrypted_data: Vec<u8> = vec![];
        encrypted_data.extend(b"PNED");
        encrypted_data.push(1);
        encrypted_data.extend(IDENTIFIER);
        encrypted_data.push(METADATA.len() as u8);
        encrypted_data.extend(METADATA);
        encrypted_data.extend(ENCRYPTED_DATA);

        let cryptor_module = CryptoModule::new(Box::new(MyCryptor), None);
        let decrypt_result = cryptor_module.decrypt(encrypted_data);

        let Ok(data) = decrypt_result else {
            panic!("Decryption should be successful")
        };

        assert_eq!(data, DECRYPTED_DATA)
    }

    #[test]
    fn not_decrypt_data_with_unknown_cryptor() {
        let mut encrypted_data: Vec<u8> = vec![];
        encrypted_data.extend(b"PNED");
        encrypted_data.push(1);
        encrypted_data.extend(b"PNDC");
        encrypted_data.push(METADATA.len() as u8);
        encrypted_data.extend(METADATA);
        encrypted_data.extend(ENCRYPTED_DATA);

        let cryptor_module = CryptoModule::new(Box::new(MyCryptor), None);
        let decrypt_result = cryptor_module.decrypt(encrypted_data);

        let Err(err) = decrypt_result else {
            panic!("Decryption should not be successful")
        };

        assert!(matches!(err, PubNubError::UnknownCryptor { .. }))
    }
}
