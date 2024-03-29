//! Publish result module.
//!
//! This module contains the `PublishResult` type.
//! The `PublishResult` type is used to represent the result of a publish operation.

use crate::{
    core::{service_response::APIErrorBody, PubNubError},
    lib::alloc::string::String,
};

/// The result of a publish operation.
/// It contains the timetoken of the published message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishResult {
    /// The timetoken of the published message.
    pub timetoken: String,
}

/// The response body of a publish operation.
/// It can be either a tuple with data from the Publish service
/// or an [`OtherResponse`] from other services.
///
/// It's used for deserialization of the publish response. This type is an
/// intermediate type between the raw response body and the [`PublishResult`]
/// type.
///
/// [`OtherResponse`]: struct.OtherResponse.html
/// [`PublishResult`]: struct.PublishResult.html
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishResponseBody {
    /// The response body of a publish operation in publish service.
    /// It contains the error indicator, the message from service and the
    /// timetoken in this order.
    ///
    /// The error indicator is `1` if the operation was successful and `0`
    /// otherwise.
    ///
    /// # Example
    /// ```json
    /// [1, "Sent", "15815800000000000"]
    /// ```
    SuccessResponse(i32, String, String),
    /// The response body of a publish operation in other services.
    ErrorResponse(APIErrorBody),
}

impl TryFrom<PublishResponseBody> for PublishResult {
    type Error = PubNubError;

    fn try_from(value: PublishResponseBody) -> Result<Self, Self::Error> {
        match value {
            PublishResponseBody::SuccessResponse(error_indicator, message, timetoken) => {
                if error_indicator == 1 {
                    Ok(PublishResult { timetoken })
                } else {
                    Err(PubNubError::general_api_error(message, None, None))
                }
            }
            PublishResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::core::TransportResponse;

    fn body_to_result(
        body: PublishResponseBody,
        response: TransportResponse,
    ) -> Result<PublishResult, PubNubError> {
        match body {
            PublishResponseBody::SuccessResponse(error_indicator, message, timetoken) => {
                if error_indicator == 1 {
                    Ok(PublishResult { timetoken })
                } else {
                    Err(PubNubError::general_api_error(
                        message,
                        Some(response.status),
                        Some(Box::new(response)),
                    ))
                }
            }
            PublishResponseBody::ErrorResponse(resp) => {
                let error: PubNubError = resp.into();
                Err(error.attach_response(response))
            }
        }
    }

    #[test]
    fn parse_publish_response() {
        let body =
            PublishResponseBody::SuccessResponse(1, "Sent".into(), "15815800000000000".into());
        let result = body_to_result(
            body,
            TransportResponse {
                status: 200,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(result.timetoken, "15815800000000000");
    }

    #[test]
    fn parse_other_response() {
        let status = 400;
        let body = PublishResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result = body_to_result(
            body,
            TransportResponse {
                status,
                ..Default::default()
            },
        );

        assert!(result.is_err());
    }
}
