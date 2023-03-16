//! This module contains the `TransportResponse` struct.
//!
//! This struct is used to represent the response from a request to the [`PubNub API`].
//! It is used as the response type for the [`Transport`] trait.
//!
//! [`Transport`]: ../transport/trait.Transport.html
//! [`PubNub API`]: https://www.pubnub.com/docs

use std::collections::HashMap;

/// This struct is used to represent the response from a request to the [`PubNub API`].
/// It is used as the response type for the [`Transport`] trait.
///
/// [`Transport`]: ../transport/trait.Transport.html
/// [`PubNub API`]: https://www.pubnub.com/docs
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportResponse {
    /// status code of the response
    pub status: u16,

    /// headers of the response
    pub headers: HashMap<String, String>,

    /// body of the response
    pub body: Option<Vec<u8>>,
}
