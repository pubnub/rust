//! Presence result module.
//!
//! This module contains the [`HeartbeatResult`] type.

use crate::core::{
    service_response::{APIErrorBody, APISuccessBodyWithMessage, APISuccessBodyWithPayload},
    PubNubError,
};

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

/// The result of a leave announcement operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeaveResult;

/// Presence service response body for leave.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaveResponseBody {
    /// This is a success response body for a announce leave operation in
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

    /// This is an error response body for a announce leave operation in the
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

impl TryFrom<LeaveResponseBody> for LeaveResult {
    type Error = PubNubError;

    fn try_from(value: LeaveResponseBody) -> Result<Self, Self::Error> {
        match value {
            LeaveResponseBody::SuccessResponse(_) => Ok(LeaveResult),
            LeaveResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

/// The result of a set state operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetStateResult {
    /// State which has been associated for `user_id` with channel(s) or channel
    /// group(s).
    #[cfg(feature = "serde")]
    state: serde_json::Value,

    /// State which has been associated for `user_id` with channel(s) or channel
    /// group(s).
    #[cfg(not(feature = "serde"))]
    state: Vec<u8>,
}

/// Set state service response body for heartbeat.
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetStateResponseBody {
    /// This is a success response body for a set state heartbeat operation in
    /// the Presence service.
    ///
    /// It contains information about the service that have the response,
    /// operation result message and state which has been associated for
    /// `user_id` with channel(s) or channel group(s).
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 200,
    ///     "message": "OK",
    ///     "payload": {
    ///         "key-1": "value-1",
    ///         "key-2": "value-2"
    ///     }
    ///     "service": "Presence"
    /// }
    /// ```
    #[cfg(feature = "serde")]
    SuccessResponse(APISuccessBodyWithPayload<serde_json::Value>),

    /// This is a success response body for a set state heartbeat operation in
    /// the Presence service.
    ///
    /// It contains information about the service that have the response,
    /// operation result message and state which has been associated for
    /// `user_id` with channel(s) or channel group(s).
    ///
    /// # Example
    /// ```json
    /// {
    ///     "status": 200,
    ///     "message": "OK",
    ///     "payload": {
    ///         "key-1": "value-1",
    ///         "key-2": "value-2"
    ///     }
    ///     "service": "Presence"
    /// }
    /// ```
    #[cfg(not(feature = "serde"))]
    SuccessResponse(APISuccessBodyWithPayload<Vec<u8>>),

    /// This is an error response body for a set state operation in the Presence
    /// service.
    ///
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

impl TryFrom<SetStateResponseBody> for SetStateResult {
    type Error = PubNubError;

    fn try_from(value: SetStateResponseBody) -> Result<Self, Self::Error> {
        match value {
            SetStateResponseBody::SuccessResponse(response) => Ok(SetStateResult {
                state: response.payload,
            }),
            SetStateResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

#[cfg(test)]
mod it_should {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn parse_heartbeat_response() {
        let body = HeartbeatResponseBody::SuccessResponse(APISuccessBodyWithMessage {
            status: 200,
            message: "OK".into(),
            service: "Presence".into(),
        });
        let result: Result<HeartbeatResult, PubNubError> = body.try_into();

        assert_eq!(result.unwrap(), HeartbeatResult);
    }

    #[test]
    fn parse_heartbeat_error_response() {
        let body = HeartbeatResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status: 400,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result: Result<HeartbeatResult, PubNubError> = body.try_into();

        assert!(result.is_err());
    }

    #[test]
    fn parse_leave_response() {
        let body = LeaveResponseBody::SuccessResponse(APISuccessBodyWithMessage {
            status: 200,
            message: "OK".into(),
            service: "Presence".into(),
        });
        let result: Result<LeaveResult, PubNubError> = body.try_into();

        assert_eq!(result.unwrap(), LeaveResult);
    }

    #[test]
    fn parse_leave_error_response() {
        let body = LeaveResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status: 400,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result: Result<LeaveResult, PubNubError> = body.try_into();

        assert!(result.is_err());
    }

    #[test]
    fn parse_set_state_response() {
        use serde_json::json;

        let payload_value = json!(HashMap::<String, String>::from([(
            "key".into(),
            "value".into()
        )]));
        let body = SetStateResponseBody::SuccessResponse(APISuccessBodyWithPayload {
            status: 200,
            message: "OK".into(),
            payload: payload_value.clone(),
            service: "Presence".into(),
        });
        let result: Result<SetStateResult, PubNubError> = body.try_into();

        assert!(payload_value.is_object());
        assert_eq!(
            result.unwrap(),
            SetStateResult {
                state: payload_value
            }
        );
    }

    #[test]
    fn parse_set_state_error_response() {
        let body = SetStateResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status: 400,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result: Result<SetStateResult, PubNubError> = body.try_into();

        assert!(result.is_err());
    }
}
