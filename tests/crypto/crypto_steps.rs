use crate::common::PubNubWorld;
use cucumber::{given, then, when};
use log::debug;
use std::fs::File;
use std::io::Read;

use pubnub::core::{CryptoProvider, Cryptor, PubNubError};
use pubnub::providers::crypto::{AesCbcCryptor, CryptoModule, LegacyCryptor};

#[given(regex = r#"^Crypto module with '(.*)' cryptor$"#)]
fn given_cryptor_with_id(world: &mut PubNubWorld, cryptor_id: String) {
    world.crypto_state.crypto_identifiers.push(cryptor_id)
}

#[given(regex = r#"^Crypto module with default '(.*)' and additional '(.*)' cryptors$"#)]
fn given_cryptor_with_multiple_id(
    world: &mut PubNubWorld,
    default_crypto_id: String,
    secondary_crypto_id: String,
) {
    world
        .crypto_state
        .crypto_identifiers
        .push(default_crypto_id);
    world
        .crypto_state
        .crypto_identifiers
        .push(secondary_crypto_id);
}

#[given(regex = r#"^with '(.*)' cipher key$"#)]
fn given_cipher_key(world: &mut PubNubWorld, cipher_key: String) {
    world.crypto_state.cipher_key = Some(cipher_key)
}

#[given(regex = r#"^with '(constant|random|-)' vector$"#)]
fn given_iv_type(world: &mut PubNubWorld, vector_type: String) {
    world.crypto_state.use_random_iv = vector_type.eq("random")
}

#[given(regex = r#"^Legacy code with '(.*)' cipher key and '(constant|random|-)' vector$"#)]
fn given_legacy_code(world: &mut PubNubWorld, cipher_key: String, vector_type: String) {
    use super::super::crypto::legacy::{AesCbcCrypto, AesCbcIv};

    let iv = if vector_type.eq("constant") {
        AesCbcIv::Constant
    } else {
        AesCbcIv::Random
    };

    world.crypto_state.legacy =
        Some(AesCbcCrypto::new(cipher_key, iv).expect("Should create legacy crypto"));
}

#[when(regex = r#"^I encrypt '(.*)' file as 'binary'$"#)]
fn when_encrypt_data(world: &mut PubNubWorld, file_name: String) {
    let cryptor_module = cryptor_module(world);
    world.crypto_state.file_content = load_file_with_name(file_name);
    world.crypto_state.encryption_result =
        Some(cryptor_module.encrypt(world.crypto_state.file_content.clone()));
}

#[when(regex = r#"^I decrypt '(.*)' file$"#)]
#[when(regex = r#"^I decrypt '(.*)' file as 'binary'$"#)]
fn when_decrypt_data(world: &mut PubNubWorld, file_name: String) {
    let cryptor_module = cryptor_module(world);
    let file_content = load_file_with_name(file_name);
    world.crypto_state.decryption_result = Some(cryptor_module.decrypt(file_content));
}

#[then(regex = r#"^I receive '(.*)'$"#)]
fn then_receive_outcome(world: &mut PubNubWorld, outcome: String) {
    let result = if world.crypto_state.encryption_result.is_some() {
        world.crypto_state.encryption_result.clone()
    } else {
        world.crypto_state.decryption_result.clone()
    };

    let Some(result) = result else {
        panic!("Expected to have result of encryption or decryption operation");
    };

    match outcome.as_str() {
        "unknown cryptor error" => {
            assert!(result.is_err(), "Operation should fail: {result:?}");
            assert!(matches!(
                result.err().unwrap(),
                PubNubError::UnknownCryptor { .. }
            ))
        }
        "decryption error" => {
            assert!(result.is_err(), "Operation should fail: {result:?}");
            assert!(matches!(
                result.err().unwrap(),
                PubNubError::Decryption { .. }
            ))
        }
        "success" => assert!(result.is_ok(), "Operation should be successful"),
        _ => panic!("Unknown outcome in feature file."),
    }
}

#[then("Successfully decrypt an encrypted file with legacy code")]
fn then_success_decrypt(world: &mut PubNubWorld) {
    use crate::crypto::legacy::cryptor::Cryptor;

    let Some(result) = &world.crypto_state.encryption_result else {
        panic!("No content has been encrypted")
    };

    assert!(result.is_ok());

    let decrypt_result = world
        .crypto_state
        .legacy
        .as_ref()
        .unwrap()
        .decrypt(result.clone().ok().unwrap())
        .expect("Should decrypt without error");

    assert_eq!(decrypt_result, world.crypto_state.file_content);
}

#[then(regex = r#"^Decrypted file content equal to the '(.*)' file content$"#)]
fn then_decrypt_data(world: &mut PubNubWorld, file_name: String) {
    let Some(result) = &world.crypto_state.decryption_result else {
        panic!("No content has been encrypted")
    };

    assert!(result.is_ok());
    assert_eq!(result.clone().ok().unwrap(), load_file_with_name(file_name));
}

fn load_file_with_name(file_name: String) -> Vec<u8> {
    let file_path = format!("tests/features/encryption/assets/{file_name}");
    let mut file = File::open(file_path).expect("File should exist!");
    let mut content: Vec<u8> = vec![];
    file.read_to_end(&mut content)
        .expect("Unable to read content");

    content
}

fn cryptor_module(world: &PubNubWorld) -> CryptoModule {
    let mut cryptors = world
        .crypto_state
        .crypto_identifiers
        .iter()
        .map(|id| {
            if id.eq("acrh") {
                Box::new(
                    AesCbcCryptor::new(world.crypto_state.cipher_key.clone().unwrap())
                        .expect("Cryptor should initialize"),
                ) as Box<dyn Cryptor>
            } else {
                Box::new(
                    LegacyCryptor::new(
                        world.crypto_state.cipher_key.clone().unwrap(),
                        world.crypto_state.use_random_iv,
                    )
                    .expect("Cryptor should initialize"),
                ) as Box<dyn Cryptor>
            }
        })
        .collect::<Vec<Box<dyn Cryptor>>>();

    CryptoModule::new(
        cryptors.remove(0),
        if cryptors.len() > 0 {
            Some(cryptors)
        } else {
            None
        },
    )
}
