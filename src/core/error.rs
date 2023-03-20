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
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
/// # Examples
/// ```
/// use pubnub::PubNubError;
///
/// fn foo() -> Result<(), PubNubError> {
///   Ok(())
/// }
///
/// foo().map_err(|e| match e {
///   PubNubError::TransportError(_) => println!("Transport error"),
///   PubNubError::PublishError(_) => println!("Publish error"),
///   _ => println!("Other error"),
/// });
///
/// ```
#[derive(thiserror::Error, Debug)]
pub enum PubNubError {
    /// this error is returned when the transport layer fails
    #[error("Transport error: {0}")]
    TransportError(String),

    /// this error is returned when the publication of the request fails
    #[error("Publish error: {0}")]
    PublishError(String),

    /// this error is returned when the serialization of the response fails
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// this error is returned when one of the needed keys is missing
    #[error("No key provided error: {0}")]
    NoKeyError(String),
}
