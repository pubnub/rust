//! # Transport Request
//!
//! This module contains the `TransportRequest` struct and related types.
//!
//! This module contains the `TransportRequest` struct and related types. It is
//! intended to be used by the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

use crate::lib::{
    alloc::{string::String, vec::Vec},
    collections::HashMap,
    core::fmt::{Display, Formatter, Result},
};

/// The method to use for a request.
///
/// This enum represents the method to use for a request. It is used by the
/// [`TransportRequest`] struct.
///
/// [`TransportRequest`]: struct.TransportRequest.html
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum TransportMethod {
    /// The GET method.
    #[default]
    Get,

    /// The POST method.
    Post,

    /// The DELETE method.
    Delete,
}

impl Display for TransportMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                TransportMethod::Get => "GET",
                TransportMethod::Post => "POST",
                TransportMethod::Delete => "DELETE",
            }
        )
    }
}

/// This struct represents a request to be sent to the PubNub API.
///
/// This struct represents a request to be sent to the PubNub API. It is used by
/// the [`Transport`] trait.
///
/// All fields are representing certain parts of the request that can be used
/// to prepare one.
///
/// [`Transport`]: ../transport/trait.Transport.html
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportRequest {
    /// path to the resource
    pub path: String,

    /// query parameters to be sent with the request
    pub query_parameters: HashMap<String, String>,

    /// method to use for the request
    pub method: TransportMethod,

    /// headers to be sent with the request
    pub headers: HashMap<String, String>,

    /// body to be sent with the request
    pub body: Option<Vec<u8>>,
}
