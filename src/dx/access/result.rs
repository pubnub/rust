//! Access manager result module.
//!
//! This module contains [`GrantTokenResult`] ans [`RevokeTokenResult`] types.
//! The [`GrantTokenResult`] type is used to represent results of access token
//! generation operation.

use crate::core::{APIErrorBody, PubNubError};

/// The result of a grant token operation.
/// It has a token that can be used to get access to restricted resources.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantTokenResult {
    /// The grant token operation was successful.
    ///
    /// The response includes a token with the requested permissions.
    pub token: String,
}

/// The result of a revoke token operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevokeTokenResult;

/// [`PubNub API`] response for grant token operation request.
///
/// Either a success response with a token from the PAMv3 service or an error
/// response with error information can be used.
/// It is used for deserializing the grant token response. This type is an
/// intermediate between the raw response body and the [`GrantTokenResult`]
/// type.
///
/// [`PubNub API`]: https://www.pubnub.com/docs
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantTokenResponseBody {
    /// This is a success response body for a grant token operation in the
    /// Access Manager service.
    /// It contains information about the service that gave the response and the
    /// data with the token that was generated.
    ///
    /// #  Example
    /// ```json
    /// {
    ///   "status": 200,
    ///   "data": {
    ///     "message": "Success",
    ///     "token": "p0F2AkF0Gl043r....Dc3BjoERtZXRhoENzaWdYIGOAeTyWGJI"
    ///   },
    ///   "service": "Access Manager"
    /// }
    /// ```
    SuccessResponse(APISuccessBody<GrantTokenPayload>),

    /// This is an error response body for a grant token operation in the
    /// Access Manager service.
    /// It contains information about the service that provided the response and
    /// details of what exactly was wrong.
    ///
    /// # Example
    /// ```json
    /// {
    ///   "status": 400,
    ///   "error": {
    ///     "message": "Invalid ttl",
    ///     "source": "grant",
    ///     "details": [
    ///       {
    ///         "message": "Range should be 1 to 43200 minute(s)",
    ///         "location": "ttl",
    ///         "locationType": "body"
    ///       }
    ///     ]
    ///   },
    ///   "service": "Access Manager"
    /// }
    /// ```
    ErrorResponse(APIErrorBody),
}

/// Service response body.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum RevokeTokenResponseBody {
    SuccessResponse(APISuccessBody<RevokeTokenPayload>),
    ErrorResponse(APIErrorBody),
}

/// Token grant operation response payload.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantTokenPayload {
    message: String,
    token: String,
}

/// Token revoke operation response payload.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevokeTokenPayload {
    message: String,
}

/// Content of successful PAMv3 REST API operation.
///
/// Body contains status code and `service` response specific to used endpoint.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct APISuccessBody<D> {
    status: i32,
    data: D,
    service: String,
}

impl TryFrom<GrantTokenResponseBody> for GrantTokenResult {
    type Error = PubNubError;

    fn try_from(value: GrantTokenResponseBody) -> Result<Self, Self::Error> {
        match value {
            GrantTokenResponseBody::SuccessResponse(resp) => Ok(GrantTokenResult {
                token: resp.data.token,
            }),
            GrantTokenResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}
