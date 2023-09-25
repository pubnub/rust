use pubnub::core::CryptoProvider;
use pubnub::providers::crypto::CryptoModule;
use pubnub::{Keyset, PubNubClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn snafu::Error>> {
    let source_data: Vec<u8> = "Hello world!".into();
    let use_random_iv = true;
    let cipher = "enigma";

    // Crypto module with legacy AES-CBC cryptor (with enhanced AES-CBC decrypt
    // support).
    let legacy_crypto_module = CryptoModule::new_legacy_module(cipher, use_random_iv)?;
    let legacy_encrypt_result = legacy_crypto_module.encrypt(source_data.clone());

    println!("encrypt with legacy AES-CBC result: {legacy_encrypt_result:?}");

    // Crypto module with enhanced AES-CBC cryptor (with legacy AES-CBC decrypt
    // support).
    let crypto_module = CryptoModule::new_aes_cbc_module(cipher, use_random_iv)?;
    let encrypt_result = crypto_module.encrypt(source_data.clone());

    println!("encrypt with enhanced AES-CBC result: {encrypt_result:?}");

    // Decrypt data created with legacy AES-CBC crypto module.
    let legacy_decrypt_result = crypto_module.decrypt(legacy_encrypt_result.ok().unwrap())?;
    assert_eq!(legacy_decrypt_result, source_data);

    // Decrypt data created with enhanced AES-CBC crypto module.
    let decrypt_result = legacy_crypto_module.decrypt(encrypt_result.ok().unwrap())?;
    assert_eq!(decrypt_result, source_data);

    // Setup client with crypto module
    let publish_key = env::var("SDK_PUB_KEY")?;
    let subscribe_key = env::var("SDK_SUB_KEY")?;

    let client = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key,
            publish_key: Some(publish_key),
            secret_key: None,
        })
        .with_user_id("user_id")
        .with_cryptor(crypto_module)
        .build()?;

    // publish encrypted string
    let result = client
        .publish_message("hello world!")
        .channel("my_channel")
        .r#type("text-message")
        .execute()
        .await?;

    println!("publish result: {:?}", result);

    Ok(())
}
