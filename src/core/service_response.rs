//! [`PubNub`] service response payload module.
//!
//! This module contains [`APISuccessBody`], [`APISuccessBodyWithMessage`] and
//! [`APIErrorBody`] types which represent result of [`PubNub`] network API
//! endpoint call.
//!
//! [`PubNub`]:https://www.pubnub.com/

use crate::{
    core::PubNubError,
    lib::{
        alloc::{
            borrow::ToOwned,
            format,
            string::{String, ToString},
            vec::Vec,
        },
        collections::HashMap,
    },
};

/// Result of successful REST API endpoint call.
///
/// Body contains status code, response `data` and `service` response specific
/// to used endpoint.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct APISuccessBody<D> {
    pub status: i32,
    pub data: D,
    pub service: String,
}

/// Result of successful REST API endpoint call.
///
/// Body contains status code, `message, response `payload` and `service`
/// response specific to used endpoint.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct APISuccessBodyWithPayload<D> {
    pub status: i32,
    pub message: String,
    pub payload: D,
    pub service: String,
}

/// Result of successful REST API endpoint call.
///
/// Body contains status code, `message` and `service` response specific to used
/// endpoint.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct APISuccessBodyWithMessage {
    pub status: i32,
    pub message: String,
    pub service: String,
}

/// Result of successful REST API endpoint call.
///
/// Body contains status code, `message` and `service` response specific to used
/// endpoint. Additionaly it contains flatten `payload` which can be any generic
/// type.
///
/// To implement deserialization for this struct - check the [`Api docs`] or docs
/// included to specific result.
///
/// [`Api docs`]: https://www.pubnub.com/docs/sdks/rest-api
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct APISuccessBodyWithFlattenedPayload<D> {
    pub status: i32,
    pub message: String,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub payload: D,
    pub service: String,
}

/// PubNub service error response.
///
/// `ErrorResponse` enum variants covers all possible [`PubNub API`] error
/// responses.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum APIErrorBody {
    /// Error response in format of an array.
    ///
    /// # Example
    /// ```json
    /// [0,"Not modified"]
    /// ```
    AsArray2(u8, String),

    /// Error response in format of an array.
    ///
    /// Such data includes information about when the error occurred.
    ///
    /// # Example
    /// ```json
    /// [0,"Signal size too large","15782702375048763"]
    /// ```
    AsArray3(u8, String, Option<String>),

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "message": "Forbidden",
    ///     "payload": {
    ///         "channels": [
    ///             "test-channel1"
    ///         ],
    ///         "channel-groups": [
    ///             "test-group1"
    ///         ]
    ///     },
    ///     "error": true,
    ///     "service": "Access Manager",
    ///     "status": 403
    /// }
    /// ```
    AsObjectWithServiceAndErrorPayload {
        /// Operation status (HTTP) code.
        status: u16,

        /// There is a flag that tells if this is an error response.
        error: bool,

        /// Service which reported an error.
        service: String,

        /// A message explaining what went wrong.
        message: String,

        /// Payload with additional information about error.
        payload: ErrorObjectPayload,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 413,
    ///     "error": true,
    ///     "service": "Balancer",
    ///     "message": "Request Entity Too Large."
    /// }
    /// ```
    AsObjectWithService {
        /// Operation status (HTTP) code.
        status: u16,

        /// There is a flag that tells if this is an error response.
        error: bool,

        /// Service which reported an error.
        service: String,

        /// A message explaining what went wrong.
        message: String,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 429,
    ///     "error": true,
    ///     "message": "Too many requests."
    /// }
    /// ```
    AsObjectWithMessage {
        /// Operation status (HTTP) code.
        status: u16,

        /// There is a flag that tells if this is an error response.
        error: bool,

        /// A message explaining what went wrong.
        message: String,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 400,
    ///     "error": true,
    ///     "error_message": "Invalid Arguments",
    ///     "channels": {}
    /// }
    /// ```
    AsObjectWithErrorMessageAndChannels {
        /// Operation status (HTTP) code.
        status: u16,

        /// There is a flag that tells if this is an error response.
        error: bool,

        /// A message explaining what went wrong.
        error_message: String,

        /// Channels that have been affected by the issue.
        ///
        /// In case of error this dictionary is empty, but present in response.
        channels: Option<HashMap<String, String>>,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 403,
    ///     "error": true,
    ///     "error_message": "Invalid Arguments"
    /// }
    /// ```
    AsObjectWithErrorMessage {
        /// Operation status (HTTP) code.
        status: u16,

        /// There is a flag that tells if this is an error response.
        error: bool,

        /// A message explaining what went wrong.
        error_message: String,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "error": {
    ///         "message": "Invalid signature",
    ///         "source": "grant",
    ///         "details": [
    ///             {
    ///                 "message": "Client and server produced different signatures for the same inputs.",
    ///                 "location": "signature",
    ///                 "locationType": "query"
    ///             }
    ///         ]
    ///     },
    ///     "service": "Access Manager",
    ///     "status": 403
    /// }
    /// ```
    AsObjectWithErrorObjectDetails {
        /// Operation status (HTTP) code.
        status: u16,

        /// Service which reported an error.
        service: String,

        /// Additional information about error.
        error: ErrorObjectWithDetails,
    },

    /// Error response in format of dictionary.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 409,
    ///     "error": {
    ///         "source": "actions",
    ///         "message": "Action Already Added"
    ///     }
    /// }
    /// ```
    AsObjectWithErrorObject {
        /// Operation status (HTTP) code.
        status: u16,

        /// Additional information about error.
        error: ErrorObject,
    },
}

/// Additional error information struct.
///
/// This structure used by [`AsObjectWithServiceAndErrorPayload`] to represent
/// list of errors in response.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErrorObjectPayload {
    /// The list of channels for which an error was reported.
    channels: Option<Vec<String>>,

    /// The list of channel groups for which an error was reported.
    #[cfg_attr(feature = "serde", serde(rename = "channel-groups"))]
    channel_groups: Option<Vec<String>>,
}

/// Additional error information struct.
///
/// This structure used by [`ErrorObjectWithDetails`] to represent list of
/// errors in response.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ErrorObjectDetails {
    /// A message explaining what went wrong.
    message: String,

    /// Which part of the request caused an issue.
    location: String,

    /// Type of issue reason.
    #[cfg_attr(feature = "serde", serde(rename(deserialize = "locationType")))]
    location_type: String,
}

/// Error description.
///
/// This structure used by [`AsObjectWithErrorObject`] to represent list of
/// errors in response.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErrorObject {
    /// A message explaining what went wrong.
    message: String,

    /// Service / sub-system which reported an error.
    source: String,
}

/// This structure used by [`APIErrorBody::AsObjectWithErrorObjectDetails`] to
/// represent server error response.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErrorObjectWithDetails {
    /// A message explaining what went wrong.
    message: String,

    /// Service / sub-system which reported an error.
    source: String,

    /// Additional information about failure reasons.
    details: Vec<ErrorObjectDetails>,
}

impl APIErrorBody {
    /// Retrieve status code from error body payload.
    fn status(&self) -> u16 {
        match self {
            APIErrorBody::AsObjectWithServiceAndErrorPayload { status, .. } => *status,
            APIErrorBody::AsObjectWithService { status, .. } => *status,
            APIErrorBody::AsObjectWithMessage { status, .. } => *status,
            APIErrorBody::AsObjectWithErrorMessageAndChannels { status, .. } => *status,
            APIErrorBody::AsObjectWithErrorMessage { status, .. } => *status,
            APIErrorBody::AsObjectWithErrorObjectDetails { status, .. } => *status,
            APIErrorBody::AsObjectWithErrorObject { status, .. } => *status,
            APIErrorBody::AsArray2(_, _) => 400,
            APIErrorBody::AsArray3(_, _, _) => 400,
        }
    }

    /// Retrieve service name from error body payload.
    fn service(&self) -> Option<String> {
        match self {
            APIErrorBody::AsObjectWithServiceAndErrorPayload { service, .. } => {
                Some(service.to_owned())
            }
            APIErrorBody::AsObjectWithErrorObjectDetails { service, .. } => {
                Some(service.to_owned())
            }
            APIErrorBody::AsObjectWithService { service, .. } => Some(service.to_owned()),
            _ => None,
        }
    }

    fn message(&self) -> String {
        match self {
            APIErrorBody::AsArray2(_, message) => message.to_owned(),
            APIErrorBody::AsArray3(_, message, _) => message.to_owned(),
            APIErrorBody::AsObjectWithServiceAndErrorPayload {
                message, payload, ..
            } => {
                let mut affected: Vec<String> = Vec::new();
                if let Some(channels) = &payload.channels {
                    if !channels.is_empty() {
                        affected.push(format!("affected channels: {}", channels.join(", ")))
                    }
                }

                if let Some(groups) = &payload.channel_groups {
                    if !groups.is_empty() {
                        affected.push(format!("affected channel groups: {}", groups.join(", ")))
                    }
                }

                if affected.is_empty() {
                    message.to_string()
                } else {
                    message.to_string() + &format!(" ({})", affected.join("; "))
                }
            }
            APIErrorBody::AsObjectWithService { message, .. } => message.to_owned(),
            APIErrorBody::AsObjectWithMessage { message, .. } => message.to_owned(),
            APIErrorBody::AsObjectWithErrorMessageAndChannels { error_message, .. } => {
                error_message.to_owned()
            }
            APIErrorBody::AsObjectWithErrorMessage { error_message, .. } => {
                error_message.to_owned()
            }
            APIErrorBody::AsObjectWithErrorObjectDetails { error, .. } => {
                let mut message = format!("{}\nDetails:\n", error.message);
                error.details.iter().for_each(|detail| {
                    message += format!(
                        "  * {} (location: '{}', name: '{}')\n",
                        detail.message, detail.location_type, detail.location
                    )
                    .as_str();
                });
                message.trim().to_string()
            }
            APIErrorBody::AsObjectWithErrorObject { error, .. } => error.message.to_string(),
        }
    }

    fn affected_channels(&self) -> Option<Vec<String>> {
        match self {
            APIErrorBody::AsObjectWithServiceAndErrorPayload { payload, .. } => {
                payload.channels.clone()
            }
            _ => None,
        }
    }

    fn affected_channel_groups(&self) -> Option<Vec<String>> {
        match &self {
            APIErrorBody::AsObjectWithServiceAndErrorPayload { payload, .. } => {
                payload.channel_groups.clone()
            }
            _ => None,
        }
    }
}

/// Implementation for [`APIError`] to create struct from service error response
/// body.
impl From<APIErrorBody> for PubNubError {
    fn from(value: APIErrorBody) -> Self {
        PubNubError::API {
            status: value.status(),
            message: value.message(),
            service: value.service(),
            affected_channels: value.affected_channels(),
            affected_channel_groups: value.affected_channel_groups(),
            response: None,
        }
    }
}
