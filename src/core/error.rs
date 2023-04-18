//! # Error types
//!
//! This module contains the error types for the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

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
///   PubNubError::Transport(_) => println!("Transport error"),
///   PubNubError::API { .. } => println!("Publish error"),
///   _ => println!("Other error"),
/// });
/// ```
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
#[derive(thiserror::Error, Debug, Clone)]
pub enum PubNubError {
    /// this error is returned when the transport layer fails
    #[error("Transport error: {0}")]
    Transport(String),

    /// this error is returned when the serialization of the response fails
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// this error is returned when the serialization of the response fails
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// this error is returned when the deserialization of the token fails
    #[cfg(feature = "parse_token")]
    #[error("Token deserialization error: {0}")]
    TokenDeserializationError(String),

    /// this error is returned when one of the needed keys is missing
    #[error("No key provided error: {0}")]
    NoKey(String),

    /// this error is returned when the initialization of client fails
    #[error("Client initialization error: {0}")]
    ClientInitialization(String),

    /// this error is returned when the initialization of the cryptor fails
    #[error("Cryptor initialization error: {0}")]
    CryptoInitialization(String),

    /// this error is returned when the cryptor is unable to decrypt data
    #[error("Data encryption error: {0}")]
    Encryption(String),

    /// this error is returned when the crypror is unable to decrypt data
    #[error("Data decryption error: {0}")]
    Decryption(String),

    ///this error is returned when REST API request can't be handled by service.
    #[error("REST API error: {message}")]
    API {
        /// Operation status (HTTP) code.
        status: u16,

        /// A message explaining what went wrong.
        message: String,

        /// Service which reported an error.
        service: Option<String>,

        /// List of channels which is affected by error.
        affected_channels: Option<Vec<String>>,

        /// List of channel groups which is affected by error.
        affected_channel_groups: Option<Vec<String>>,
    },
}

impl PubNubError {
    /// Create general API call error.
    ///
    /// This function used to inform about not initialized request parameters or
    /// validation failure.
    pub(crate) fn general_api_error<S>(message: S, status: Option<u16>) -> Self
    where
        S: Into<String>,
    {
        Self::API {
            status: status.unwrap_or(400),
            message: message.into(),
            service: None,
            affected_channels: None,
            affected_channel_groups: None,
        }
    }
}
