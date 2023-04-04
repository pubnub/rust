//! Publish result module.
//!
//! This module contains the `PublishResult` type.
//! The `PublishResult` type is used to represent the result of a publish operation.

use crate::core::PubNubError;

/// The result of a publish operation.
/// It contains the timetoken of the published message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishResult {
    /// The timetoken of the published message.
    pub timetoken: String,
}

/// The response body of a publish operation.
/// It can be either a tuple with data from Publish service
/// or an [`OtherResponse`] from other services.
///
/// It used for deserialization of publish response. This type is intermediate
/// type between the raw response body and the [`PublishResult`] type.
///
/// [`OtherResponse`]: struct.OtherResponse.html
/// [`PublishResult`]: struct.PublishResult.html
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PublishResponseBody {
    /// The response body of a publish operation in publish service.
    /// It contains the error indicator, the message from service and the timetoken
    /// in this order.
    ///
    /// The error indicator is `1` if the operation was successful and `0` otherwise.
    ///
    /// # Example
    /// ```json
    /// [1, "Sent", "15815800000000000"]
    /// ```
    PublishResponse(i32, String, String),
    /// The response body of a publish operation in other services.
    OtherResponse(OtherResponse),
}

/// The response body of a publish operation in other services.
/// It contains the status code, the error indicator, the service name and the message.
/// The error indicator is `true` if the operation was successful and `false` otherwise.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OtherResponse {
    /// Status code of the response.
    pub status: u16,

    /// Indicates if the operation was successful.
    pub error: bool,

    /// The name of the service.
    pub service: String,

    /// The message from the service.
    pub message: String,
}

pub(super) fn body_to_result(
    body: PublishResponseBody,
    status: u16,
) -> Result<PublishResult, PubNubError> {
    match body {
        PublishResponseBody::PublishResponse(error_indicator, message, timetoken) => {
            if error_indicator == 1 {
                Ok(PublishResult { timetoken })
            } else {
                Err(PubNubError::PublishError(format!(
                    "Status code: {}, body: {:?}",
                    status, message
                )))
            }
        }
        PublishResponseBody::OtherResponse(body) => Err(PubNubError::PublishError(format!(
            "Status code: {}, body: {:?}",
            status, body
        ))),
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn parse_publish_response() {
        let body = PublishResponseBody::PublishResponse(
            1,
            "Sent".to_string(),
            "15815800000000000".to_string(),
        );
        let result = body_to_result(body, 200).unwrap();

        assert_eq!(result.timetoken, "15815800000000000");
    }

    #[test]
    fn parse_other_response() {
        let status = 400;

        let body = PublishResponseBody::OtherResponse(OtherResponse {
            status,
            error: true,
            service: "service".to_string(),
            message: "error".to_string(),
        });
        let result = body_to_result(body, status);

        assert!(result.is_err());
    }
}
