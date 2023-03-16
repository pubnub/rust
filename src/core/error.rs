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
/// use pubnub::error::PubNubError;
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

    /// this error is returned when the serialization of the request fails
    #[error("Publish error: {0}")]
    PublishError(String),
}
