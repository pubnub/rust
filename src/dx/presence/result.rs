//! Presence result module.
//!
//! This module contains the [`HeartbeatResult`] type.

use crate::{
    core::{
        service_response::{
            APIErrorBody, APISuccessBodyWithFlattenedPayload, APISuccessBodyWithMessage,
            APISuccessBodyWithPayload,
        },
        PubNubError,
    },
    lib::{collections::HashMap, core::ops::Deref},
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

/// The result of a here now operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNowResult {
    /// Here now channels.
    pub channels: Vec<HereNowChannel>,
    /// Total channels in the result.
    pub total_channels: u32,
    /// Amount of all users in all provided channels.
    pub total_occupancy: u32,
}

/// The here now channel data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNowChannel {
    /// Name of the channel
    pub name: String,

    /// Amount of users in the channel
    pub occupancy: u32,

    /// Users data
    pub occupants: Vec<HereNowUser>,
}

/// The here now user data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNowUser {
    /// User's id
    pub user_id: String,

    /// State defined for the user
    #[cfg(feature = "serde")]
    pub state: Option<serde_json::Value>,

    /// State defined for the user
    #[cfg(not(feature = "serde"))]
    pub state: Option<Vec<u8>>,
}

/// Here now service response body for here now.
/// This is a success response body for a here now  operation in The
/// Presence service.
///
/// It contains information about the success of the operation, the service that
/// provided the response, and the result of the operation.
///
/// Also it can contain information about the occupancy of the channel(s) Or
/// channel group(s) and the list of user id(s) subscribed.
///
/// Additionally, it can provide error information if the operation failed.
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HereNowResponseBody {
    /// This is a success response body for a here now operation in the
    /// Presence service.
    ///
    /// It contains information about the success of the operation, the service
    /// that provided the response, and the result of the operation.
    SuccessResponse(HereNowResponseSuccessBody),

    /// This is an error response body for a here now operation in the Presence
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

/// Possible variants of HereNow service success response body.
///
/// Check [HereNowResponseBody] for more details.
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HereNowResponseSuccessBody {
    /// Single channel
    ///
    /// # Example
    ///
    /// without state:
    ///
    /// ```json
    /// {
    ///    "status":200,
    ///    "message":"OK",
    ///    "occupancy":1,
    ///    "uuids":[
    ///       "just_me"
    ///    ],
    ///    "service":"Presence"
    /// }
    /// ```
    ///
    /// with state:
    ///
    /// ```json
    /// {
    ///    "message": "OK",
    ///    "occupancy": 3,
    ///    "service": "Presence",
    ///    "status": 200,
    ///    "uuids": [
    ///        {
    ///            "state": {
    ///                "channel1-state": [
    ///                    "channel-1-random-value"
    ///                ]
    ///            },
    ///            "uuid": "Earline"
    ///        },
    ///        {
    ///            "state": {
    ///                "channel1-state": [
    ///                    "channel-1-random-value"
    ///                ]
    ///            },
    ///            "uuid": "Glen"
    ///        }
    ///    ]
    /// }
    /// ```
    SingleChannel(APISuccessBodyWithFlattenedPayload<HereNowResponseChannelIdentifier>),

    /// Multiple channels
    ///
    /// # Example
    ///
    /// without state:
    /// ```json
    /// {
    ///   "status":200,
    ///   "message":"OK",
    ///   "payload":{
    ///      "channels":{
    ///         "my_channel":{
    ///            "occupancy":1,
    ///            "uuids":[
    ///               "pn-200543f2-b394-4909-9e7b-987848e44729"
    ///            ]
    ///         },
    ///         "kekw":{
    ///            "occupancy":1,
    ///            "uuids":[
    ///               "just_me"
    ///            ]
    ///         }
    ///      },
    ///      "total_channels":2,
    ///      "total_occupancy":2
    ///   },
    ///   "service":"Presence"
    /// }
    /// ```
    ///
    /// with state:
    /// ```json
    /// {
    ///    "message": "OK",
    ///    "payload": {
    ///        "channels": {
    ///            "test-channel1": {
    ///                "occupancy": 1,
    ///                "uuids": [
    ///                    {
    ///                        "state": {
    ///                            "channel1-state": [
    ///                                "channel-1-random-value"
    ///                            ]
    ///                        },
    ///                        "uuid": "Kim"
    ///                    }
    ///                ]
    ///            },
    ///            "test-channel2": {
    ///                "occupancy": 1,
    ///                "uuids": [
    ///                    {
    ///                        "state": {
    ///                            "channel2-state": [
    ///                                "channel-2-random-value"
    ///                            ]
    ///                        },
    ///                        "uuid": "Earline"
    ///                    }
    ///                ]
    ///            }
    ///        },
    ///        "total_channels": 3,
    ///        "total_occupancy": 3
    ///    },
    ///    "service": "Presence",
    ///    "status": 200
    /// }
    /// ```
    MultipleChannels(APISuccessBodyWithPayload<HereNowResponseChannels>),
}

/// Channels in here now response.
///
/// Check [`HereNowResponseSuccessBody`] for more details.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNowResponseChannels {
    /// Amount of channels in response.
    pub total_channels: u32,

    /// Amount of users in response.
    pub total_occupancy: u32,

    /// List of channels in response.
    pub channels: HashMap<String, HereNowResponseChannelIdentifier>,
}

/// Channel description in here now response.
///
/// Check [`HereNowResponseSuccessBody`] for more details.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereNowResponseChannelIdentifier {
    /// Amount of users in channel.
    pub occupancy: u32,

    /// List of users in channel.
    pub uuids: Option<Vec<HereNowResponseUserIdentifier>>,
}

/// Possible variants of user identifier in here now response.
///
/// Check [`HereNowResponseSuccessBody`] for more details.
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HereNowResponseUserIdentifier {
    /// User identifier is a string.
    String(String),

    /// User identifier is a map of channel names to their states.
    WithState {
        /// User identifier.
        uuid: String,

        /// User state.
        #[cfg(feature = "serde")]
        state: serde_json::Value,

        /// User state.
        #[cfg(not(feature = "serde"))]
        state: Vec<u8>,
    },

    /// User identifier is a map of uuids
    Map {
        /// User identifier.
        uuid: String,
    },
}

impl TryFrom<HereNowResponseBody> for HereNowResult {
    type Error = PubNubError;

    fn try_from(value: HereNowResponseBody) -> Result<Self, Self::Error> {
        match value {
            HereNowResponseBody::SuccessResponse(resp) => Ok(match resp {
                HereNowResponseSuccessBody::SingleChannel(single) => {
                    let occupancy = single.payload.occupancy;

                    let occupants = single
                        .payload
                        .uuids
                        .map(|maybe_uuids| {
                            maybe_uuids
                                .iter()
                                .map(|uuid| match uuid {
                                    HereNowResponseUserIdentifier::String(uuid) => HereNowUser {
                                        user_id: uuid.clone(),
                                        state: None,
                                    },
                                    HereNowResponseUserIdentifier::Map { uuid } => HereNowUser {
                                        user_id: uuid.clone(),
                                        state: None,
                                    },
                                    HereNowResponseUserIdentifier::WithState { uuid, state } => {
                                        HereNowUser {
                                            user_id: uuid.clone(),
                                            state: Some(state.clone()),
                                        }
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let channels = vec![HereNowChannel {
                        name: "".into(),
                        occupancy,
                        occupants,
                    }];

                    Self {
                        channels,
                        total_channels: 1,
                        total_occupancy: occupancy,
                    }
                }
                HereNowResponseSuccessBody::MultipleChannels(multiple) => {
                    let total_channels = multiple.payload.total_channels;
                    let total_occupancy = multiple.payload.total_occupancy;

                    let channels = multiple
                        .payload
                        .channels
                        .into_iter()
                        .map(|(name, channel)| {
                            let occupancy = channel.occupancy;

                            let occupants = channel
                                .uuids
                                .map(|maybe_uuids| {
                                    maybe_uuids
                                        .into_iter()
                                        .map(|uuid| match uuid {
                                            HereNowResponseUserIdentifier::String(uuid) => {
                                                HereNowUser {
                                                    user_id: uuid.clone(),
                                                    state: None,
                                                }
                                            }
                                            HereNowResponseUserIdentifier::Map { uuid } => {
                                                HereNowUser {
                                                    user_id: uuid.clone(),
                                                    state: None,
                                                }
                                            }
                                            HereNowResponseUserIdentifier::WithState {
                                                uuid,
                                                state,
                                            } => HereNowUser {
                                                user_id: uuid.clone(),
                                                state: Some(state.clone()),
                                            },
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            HereNowChannel {
                                name,
                                occupancy,
                                occupants,
                            }
                        })
                        .collect();

                    Self {
                        channels,
                        total_channels,
                        total_occupancy,
                    }
                }
            }),
            HereNowResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

impl Deref for HereNowResult {
    type Target = Vec<HereNowChannel>;

    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

/// The result of a here now operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereNowResult {
    /// Here now channels.
    pub channels: Vec<String>,
}

/// Where now service response body for where now.
/// This is a success response body for a where now  operation in The
/// Presence service.
///
/// It contains information about the success of the operation, the service that
/// provided the response, and the result of the operation.
///
/// It also contains information about the channels that the user is currently
/// subscribed to.
///
/// Additionally, it can provide error information if the operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
pub enum WhereNowResponseBody {
    /// This is a success response body for a where now operation in the
    /// Presence service.
    ///
    /// It contains information about the success of the operation, the service
    /// that provided the response, and the result of the operation.
    SuccessResponse(APISuccessBodyWithPayload<WhereNowResponseSuccessBody>),

    /// This is an error response body for a where now operation in the Presence
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

/// The result of a where now operation.
///
/// # Example
/// ```json
/// {
///   "status":200,
///   "message":"OK",
///   "payload":{
///      "channels":[
///         "my_channel"
///      ]
///   },
///   "service":"Presence"
/// }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereNowResponseSuccessBody {
    /// Channels that the user is currently subscribed to.
    pub channels: Vec<String>,
}

impl TryFrom<WhereNowResponseBody> for WhereNowResult {
    type Error = PubNubError;

    fn try_from(value: WhereNowResponseBody) -> Result<Self, Self::Error> {
        match value {
            WhereNowResponseBody::SuccessResponse(resp) => Ok(Self {
                channels: resp.payload.channels,
            }),
            WhereNowResponseBody::ErrorResponse(resp) => Err(resp.into()),
        }
    }
}

impl Deref for WhereNowResult {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.channels
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

    #[test]
    fn parse_here_now_response_single_channel() {
        use serde_json::json;

        let input = json!({
           "status":200,
           "message":"OK",
           "occupancy":1,
           "uuids":[
              "just_me"
           ],
           "service":"Presence"
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 1);
        assert_eq!(result.total_occupancy, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap().name, "");
        assert_eq!(result.first().unwrap().occupancy, 1);
        assert_eq!(
            result.first().unwrap().occupants.first().unwrap().user_id,
            "just_me"
        );
        assert_eq!(
            result.first().unwrap().occupants.first().unwrap().state,
            None
        );
    }

    #[test]
    fn parse_here_now_response_single_channel_with_state() {
        use serde_json::json;

        let input = json!({
           "message": "OK",
           "occupancy": 2,
           "service": "Presence",
           "status": 200,
           "uuids": [
               {
                   "state": {
                       "channel1-state": [
                           "channel-1-random-value"
                       ]
                   },
                   "uuid": "Earline"
               },
               {
                   "state": {
                       "channel1-state": [
                           "channel-1-random-value"
                       ]
                   },
                   "uuid": "Glen"
               }
           ]
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 1);
        assert_eq!(result.total_occupancy, 2);
        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|channel| channel.name.is_empty()));
        assert!(result.iter().any(|channel| channel.occupancy == 2));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().user_id == "Earline"));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().state
                == Some(json!({"channel1-state": ["channel-1-random-value"]}))));
    }

    #[test]
    fn parse_here_now_response_multiple_channels() {
        use serde_json::json;

        let input = json!({
               "status":200,
               "message":"OK",
               "payload":{
                  "channels":{
                     "my_channel":{
                        "occupancy":1,
                        "uuids":[
                           "pn-200543f2-b394-4909-9e7b-987848e44729"
                        ]
                     },
                     "kekw":{
                        "occupancy":1,
                        "uuids":[
                           "just_me"
                        ]
                     }
                  },
                  "total_channels":2,
                  "total_occupancy":2
               },
               "service":"Presence"
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 2);
        assert_eq!(result.total_occupancy, 2);
        assert_eq!(result.len(), 2);

        assert!(result.iter().any(|channel| channel.name == "my_channel"));
        assert!(result.iter().any(|channel| channel.occupancy == 1));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().user_id
                == "pn-200543f2-b394-4909-9e7b-987848e44729"));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().state.is_none()));
    }

    #[test]
    fn parse_here_now_response_multiple_channels_with_state() {
        use serde_json::json;

        let input = json!({
        "message": "OK",
        "payload": {
            "channels": {
                "test-channel1": {
                    "occupancy": 1,
                    "uuids": [
                        {
                            "state": {
                                "channel1-state": [
                                    "channel-1-random-value"
                                ]
                            },
                            "uuid": "Kim"
                        }
                    ]
                },
                "test-channel2": {
                    "occupancy": 1,
                    "uuids": [
                        {
                            "state": {
                                "channel2-state": [
                                    "channel-2-random-value"
                                ]
                            },
                            "uuid": "Earline"
                        }
                    ]
                }
            },
            "total_channels": 2,
            "total_occupancy": 2
        },
        "service": "Presence",
        "status": 200
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 2);
        assert_eq!(result.total_occupancy, 2);
        assert_eq!(result.len(), 2);

        assert!(result.iter().any(|channel| channel.name == "test-channel1"));
        assert!(result.iter().any(|channel| channel.occupancy == 1));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().user_id == "Kim"));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().state
                == Some(json!({"channel1-state": ["channel-1-random-value"]}))));
    }

    #[test]
    fn parse_here_now_response_single_channel_with_map_uuid() {
        use serde_json::json;

        let input = json!({
           "status":200,
           "message":"OK",
           "occupancy":1,
           "uuids":[
               {"uuid":"just_me"}
           ],
           "service":"Presence"
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 1);
        assert_eq!(result.total_occupancy, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap().name, "");
        assert_eq!(result.first().unwrap().occupancy, 1);
        assert_eq!(
            result.first().unwrap().occupants.first().unwrap().user_id,
            "just_me"
        );
        assert_eq!(
            result.first().unwrap().occupants.first().unwrap().state,
            None
        );
    }

    #[test]
    fn parse_here_now_response_multiple_channels_with_map_uuid() {
        use serde_json::json;

        let input = json!({
               "status":200,
               "message":"OK",
               "payload":{
                  "channels":{
                     "my_channel":{
                        "occupancy":1,
                        "uuids":[
                            {"uuid":"pn-200543f2-b394-4909-9e7b-987848e44729"}
                        ]
                     },
                     "kekw":{
                        "occupancy":1,
                        "uuids":[
                            {"uuid":"just_me"}
                        ]
                     }
                  },
                  "total_channels":2,
                  "total_occupancy":2
               },
               "service":"Presence"
        });

        let result: HereNowResult = serde_json::from_value::<HereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result.total_channels, 2);
        assert_eq!(result.total_occupancy, 2);
        assert_eq!(result.len(), 2);

        assert!(result.iter().any(|channel| channel.name == "my_channel"));
        assert!(result.iter().any(|channel| channel.occupancy == 1));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().user_id
                == "pn-200543f2-b394-4909-9e7b-987848e44729"));
        assert!(result
            .iter()
            .any(|channel| channel.occupants.first().unwrap().state.is_none()));
    }

    #[test]
    fn parse_here_now_error_response() {
        let body = HereNowResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status: 400,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result: Result<HereNowResult, PubNubError> = body.try_into();

        assert!(result.is_err());
    }

    #[test]
    fn parse_where_now_response() {
        use serde_json::json;

        let input = json!({
           "status":200,
           "message":"OK",
           "payload":{
              "channels":[
                 "my_channel"
              ]
           },
           "service":"Presence"
        });

        let result: WhereNowResult = serde_json::from_value::<WhereNowResponseBody>(input)
            .unwrap()
            .try_into()
            .unwrap();

        result
            .channels
            .iter()
            .any(|channel| channel == "my_channel");
    }

    #[test]
    fn parse_where_now_error_response() {
        let body = WhereNowResponseBody::ErrorResponse(APIErrorBody::AsObjectWithService {
            status: 400,
            error: true,
            service: "service".into(),
            message: "error".into(),
        });
        let result: Result<WhereNowResult, PubNubError> = body.try_into();

        assert!(result.is_err());
    }
}
