//! # Error types
//!
//! This module contains the error types for the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

use crate::{
    core::TransportResponse,
    lib::alloc::{boxed::Box, string::String, vec::Vec},
};
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
///   PubNubError::Transport { .. } => println!("Transport error"),
///   PubNubError::API { .. } => println!("Publish error"),
///   _ => println!("Other error"),
/// });
/// ```
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
#[derive(Snafu, Debug, Clone, PartialEq)]
pub enum PubNubError {
    /// this error is returned when the transport layer fails
    #[snafu(display("Transport error: {details}"))]
    Transport {
        ///docs
        details: String,

        /// Failed request HTTP status code.
        response: Option<Box<TransportResponse>>,
    },

    /// this error is returned when the publication of the request fails
    #[snafu(display("Publish error: {details}"))]
    PublishError {
        ///docs
        details: String,
    },

    /// this error is returned when the serialization of the response fails
    #[snafu(display("Serialization error: {details}"))]
    Serialization {
        ///docs
        details: String,
    },

    /// this error is returned when the serialization of the response fails
    #[snafu(display("Deserialization error: {details}"))]
    Deserialization {
        ///docs
        details: String,
    },

    /// this error is returned when the deserialization of the token fails
    #[cfg(feature = "parse_token")]
    #[snafu(display("Token deserialization error: {details}"))]
    TokenDeserialization {
        ///docs
        details: String,
    },

    /// this error is returned when one of the needed keys is missing
    #[snafu(display("No key provided error: {details}"))]
    NoKey {
        ///docs
        details: String,
    },

    /// this error is returned when the initialization of client fails
    #[snafu(display("Client initialization error: {details}"))]
    ClientInitialization {
        ///docs
        details: String,
    },

    /// this error is returned when the initialization of the cryptor fails
    #[snafu(display("Crypto initialization error: {details}"))]
    CryptoInitialization {
        ///docs
        details: String,
    },

    /// this error is returned when the cryptor is unable to encrypt data
    #[snafu(display("Data encryption error: {details}"))]
    Encryption {
        ///docs
        details: String,
    },

    /// this error is returned when the cryptor is unable to decrypt data
    #[snafu(display("Data decryption error: {details}"))]
    Decryption {
        ///docs
        details: String,
    },

    /// this error returned when suitable cryptor not found for data decryption.
    #[snafu(display("Unknown cryptor error: {details}"))]
    UnknownCryptor {
        /// docs
        details: String,
    },

    /// this error is returned when the event engine effect is canceled
    #[snafu(display("Event engine effect has been canceled"))]
    EffectCanceled,

    /// this error is returned when the subscription initialization fails
    #[snafu(display("Subscription initialization error: {details}"))]
    SubscribeInitialization {
        ///docs
        details: String,
    },

    ///this error is returned when REST API request can't be handled by
    /// service.
    #[snafu(display("REST API error: {message}"))]
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

        /// Raw service response.
        response: Option<Box<TransportResponse>>,
    },
}

impl PubNubError {
    /// Create general API call error.
    ///
    /// This function used to inform about not initialized request parameters or
    /// validation failure.
    #[cfg(any(feature = "publish", feature = "access", feature = "subscribe"))]
    pub(crate) fn general_api_error<S>(
        message: S,
        status: Option<u16>,
        response: Option<Box<TransportResponse>>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::API {
            status: status.unwrap_or(400),
            message: message.into(),
            service: None,
            affected_channels: None,
            affected_channel_groups: None,
            response,
        }
    }

    /// Retrieve attached service response.
    #[cfg(all(
        feature = "std",
        any(feature = "publish", feature = "access", feature = "subscribe")
    ))]
    pub(crate) fn transport_response(&self) -> Option<Box<TransportResponse>> {
        match self {
            PubNubError::API { response, .. } | PubNubError::Transport { response, .. } => {
                response.clone()
            }
            _ => None,
        }
    }

    /// Attach service response.
    ///
    /// For better understanding some errors may provide additional information
    /// right from service response.
    #[cfg(any(feature = "publish", feature = "access", feature = "subscribe"))]
    pub(crate) fn attach_response(self, service_response: TransportResponse) -> Self {
        match &self {
            PubNubError::API {
                status,
                message,
                service,
                affected_channels,
                affected_channel_groups,
                ..
            } => PubNubError::API {
                status: *status,
                message: message.clone(),
                service: service.clone(),
                affected_channels: affected_channels.clone(),
                affected_channel_groups: affected_channel_groups.clone(),
                response: Some(Box::new(service_response)),
            },
            PubNubError::Transport { details, .. } => PubNubError::Transport {
                details: details.clone(),
                response: Some(Box::new(service_response)),
            },
            _ => self,
        }
    }
}
