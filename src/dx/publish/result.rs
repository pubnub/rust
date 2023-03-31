//! Publish result module.
//!
//! This module contains the `PublishResult` type.
//! The `PublishResult` type is used to represent the result of a publish operation.

/// The result of a publish operation.
/// It contains the timetoken of the published message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublishResult {
    /// The timetoken of the published message.
    pub timetoken: String,
}

/// The response body of a publish operation.
/// It can be either a [`PublishResponse`] or an [`OtherResponse`].
///
/// It used for deserialization of publish response. This type is intermediate
/// type between the raw response body and the [`PublishResult`] type.
///
/// [`PublishResponse`]: struct.PublishResponse.html
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
