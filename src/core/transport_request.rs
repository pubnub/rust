//! # Transport Request
//!
//! This module contains the `TransportRequest` struct and related types.
//!
//! This module contains the `TransportRequest` struct and related types. It is
//! intended to be used by the [`pubnub`] crate.
//!
//! [`pubnub`]: ../index.html

use crate::{
    core::PubNubError,
    lib::{
        alloc::{
            boxed::Box,
            sync::Arc,
            {string::String, vec::Vec},
        },
        collections::HashMap,
        core::fmt::{Display, Formatter},
    },
};

type DeserializerClosure<B> = Box<dyn FnOnce(&[u8]) -> Result<B, PubNubError>>;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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

impl TransportRequest {
    /// Send async request and process [`PubNub API`] response.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(not(feature = "serde"))]
    pub(crate) async fn send<B, R, T, D>(
        &self,
        transport: &T,
        deserializer: Arc<D>,
    ) -> Result<R, PubNubError>
    where
        B: for<'de> super::Deserialize<'de>,
        R: TryFrom<B, Error = PubNubError>,
        T: super::Transport,
        D: super::Deserializer + 'static,
    {
        // Request configured endpoint.
        let response = transport.send(self.clone()).await?;
        Self::deserialize(
            response.clone(),
            Box::new(move |bytes| deserializer.deserialize(bytes)),
        )
    }

    /// Send async request and process [`PubNub API`] response.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(feature = "serde")]
    pub(crate) async fn send<B, R, T, D>(
        &self,
        transport: &T,
        deserializer: Arc<D>,
    ) -> Result<R, PubNubError>
    where
        B: for<'de> serde::Deserialize<'de>,
        R: TryFrom<B, Error = PubNubError>,
        T: super::Transport,
        D: super::Deserializer + 'static,
    {
        // Request configured endpoint.
        let response = transport.send(self.clone()).await?;

        Self::deserialize(
            response.clone(),
            Box::new(move |bytes| deserializer.deserialize(bytes)),
        )
    }
    /// Send async request and process [`PubNub API`] response.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(not(feature = "serde"), feature = "blocking"))]
    pub(crate) fn send_blocking<B, R, T, D>(
        &self,
        transport: &T,
        deserializer: Arc<D>,
    ) -> Result<R, PubNubError>
    where
        B: for<'de> super::Deserialize<'de>,
        R: TryFrom<B, Error = PubNubError>,
        T: super::blocking::Transport,
        D: super::Deserializer + 'static,
    {
        // Request configured endpoint.
        let response = transport.send(self.clone())?;
        Self::deserialize(
            response.clone(),
            Box::new(move |bytes| deserializer.deserialize(bytes)),
        )
    }

    /// Send blocking request and process [`PubNub API`] response.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    #[cfg(all(feature = "serde", feature = "blocking"))]
    pub(crate) fn send_blocking<B, R, T, D>(
        &self,
        transport: &T,
        deserializer: Arc<D>,
    ) -> Result<R, PubNubError>
    where
        B: for<'de> serde::Deserialize<'de>,
        R: TryFrom<B, Error = PubNubError>,
        T: super::blocking::Transport,
        D: super::Deserializer + 'static,
    {
        // Request configured endpoint.
        let response = transport.send(self.clone())?;
        Self::deserialize(
            response.clone(),
            Box::new(move |bytes| deserializer.deserialize(bytes)),
        )
    }

    /// Deserialize [`PubNub API`] response.
    ///
    /// [`PubNub API`]: https://www.pubnub.com/docs
    fn deserialize<B, R>(
        response: super::TransportResponse,
        des: DeserializerClosure<B>,
    ) -> Result<R, PubNubError>
    where
        R: TryFrom<B, Error = PubNubError>,
    {
        response
            .clone()
            .body
            .map(|bytes| {
                let deserialize_result = des(&bytes);
                if deserialize_result.is_err() && response.status >= 500 {
                    Err(PubNubError::general_api_error(
                        "Unexpected service response",
                        None,
                        Some(Box::new(response.clone())),
                    ))
                } else {
                    deserialize_result
                }
            })
            .map_or(
                Err(PubNubError::general_api_error(
                    "No body in the response!",
                    None,
                    Some(Box::new(response.clone())),
                )),
                |response_body| {
                    response_body.and_then::<R, _>(|body: B| {
                        body.try_into().map_err(|response_error: PubNubError| {
                            response_error.attach_response(response)
                        })
                    })
                },
            )
    }
}
