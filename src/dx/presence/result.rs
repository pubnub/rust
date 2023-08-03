use crate::core::service_response::APISuccessBodyWithMessage;
use crate::core::{APIErrorBody, PubNubError};

/// The result of a heartbeat announcement operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeartbeatResult;

/// Presence service response body for heartbeat.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatResponseBody {
    /// This is a success response body for a announce heartbeat operation in
    /// the Presence service.
    ///
    /// It contains information about the service that have the response and
    /// operation result message.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 200,
    ///     "message": "OK",
    ///     "service": "Presence"
    /// }
    /// ```
    SuccessResponse(APISuccessBodyWithMessage),

    /// This is an error response body for a announce heartbeat operation in the
    /// Presence service.
    /// It contains information about the service that provided the response and
    /// details of what exactly was wrong.
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
    ErrorResponse(APIErrorBody),
}

impl TryFrom<HeartbeatResponseBody> for HeartbeatResult {
    type Error = PubNubError;

    fn try_from(value: HeartbeatResponseBody) -> Result<Self, Self::Error> {
        match value {
            HeartbeatResponseBody::SuccessResponse(_) => Ok(HeartbeatResult),
            HeartbeatResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}
