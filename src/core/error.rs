//! # Error types
//!
//! This module contains the error types for the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

use crate::lib::a::string::String;
use snafu::Snafu;

/// PubNub error type
///
/// This type is used to represent errors that can occur in the PubNub protocol.
/// It is used as the error type for the [`Result`] type.
///
/// # Examples
/// ```
/// use pubnub::core::PubNubError;
///
/// fn foo() -> Result<(), PubNubError> {
///   Ok(())
/// }
///
/// foo().map_err(|e| match e {
///   PubNubError::TransportError{details: _} => println!("Transport error"),
///   PubNubError::PublishError{details: _}  => println!("Publish error"),
///   _ => println!("Other error"),
/// });
/// ```
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
#[derive(Snafu, Debug)]
pub enum PubNubError {
    /// this error is returned when the transport layer fails
    #[snafu(display("Transport error: {details}"))]
    TransportError {
        ///docs
        details: String,
    },

    /// this error is returned when the publication of the request fails
    #[snafu(display("Publish error: {details}"))]
    PublishError {
        ///docs
        details: String,
    },

    /// this error is returned when the serialization of the response fails
    #[snafu(display("Serialization error: {details}"))]
    SerializationError {
        ///docs
        details: String,
    },

    /// this error is returned when the serialization of the response fails
    #[snafu(display("Deserialization error: {details}"))]
    DeserializationError {
        ///docs
        details: String,
    },

    /// this error is returned when the deserialization of the token fails
    #[cfg(feature = "parse_token")]
    #[snafu(display("Token deserialization error: {details}"))]
    TokenDeserializationError {
        ///docs
        details: String,
    },

    /// this error is returned when one of the needed keys is missing
    #[snafu(display("No key provided error: {details}"))]
    NoKeyError {
        ///docs
        details: String,
    },

    /// this error is returned when the initialization of client fails
    #[snafu(display("Client initialization error: {details}"))]
    ClientInitializationError {
        ///docs
        details: String,
    },

    /// this error is returned when the initialization of the cryptor fails
    #[snafu(display("Cryptor initialization error: {details}"))]
    CryptoInitializationError {
        ///docs
        details: String,
    },

    /// this error is returned when the cryptor is unable to decrypt data
    #[snafu(display("Data encryption error: {details}"))]
    EncryptionError {
        ///docs
        details: String,
    },

    /// this error is returned when the crypror is unable to decrypt data
    #[snafu(display("Data decryption error: {details}"))]
    DecryptionError {
        ///docs
        details: String,
    },
}
