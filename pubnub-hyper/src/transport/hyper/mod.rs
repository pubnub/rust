//! Hyper transport implementation.

use crate::core::data::{
    message::{Message, Type},
    request, response,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::{Transport, TransportService};
use async_trait::async_trait;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use pubnub_util::encoded_channels_list::EncodedChannelsList;

use derive_builder::Builder;
use futures_util::stream::StreamExt;
use hyper::{client::HttpConnector, Body, Client, Response, Uri};
use hyper_tls::HttpsConnector;
use std::time::Duration;

pub mod error;
pub mod presence;
pub mod pubsub;

mod util;

type HttpClient = Client<HttpsConnector<HttpConnector>>;

/// Implements transport for PubNub using the `hyper` crate to communicate with
/// the PubNub REST API.
#[derive(Debug, Clone, Builder)]
pub struct Hyper {
    /// An HTTP client to use.
    #[builder(default = "Self::default_http_client()")]
    http_client: HttpClient,

    /// Subscribe key to use in requests.
    #[builder(setter(into))]
    subscribe_key: String,
    /// Publish key to use in requests.
    #[builder(setter(into))]
    publish_key: String,

    /// The authority URL part to use to connet to the PubNub edge network
    #[builder(setter(into), default = "\"ps.pndsn.com\".to_owned()")]
    origin: String,
    /// User-Agent header value to use at HTTP requests.
    #[builder(setter(into), default = "\"Rust-Agent\".to_owned()")]
    agent: String,
}

impl Hyper {
    /// Produces a builder that can be used to construct [`Hyper`] transport.
    #[must_use]
    #[allow(clippy::new_ret_no_self)] // builder pattern should be detected
    pub fn new() -> HyperBuilder {
        HyperBuilder::default()
    }

    fn build_uri(&self, path_and_query: &str) -> Result<Uri, http::Error> {
        let url = Uri::builder()
            .scheme("https")
            .authority(self.origin.as_str())
            .path_and_query(path_and_query)
            .build()?;
        debug!("URL: {}", url);
        Ok(url)
    }
}

impl Transport for Hyper {
    type Error = error::Error;
}

impl HyperBuilder {
    fn default_http_client() -> HttpClient {
        let https = HttpsConnector::new();
        Client::builder()
            .keep_alive_timeout(Some(Duration::from_secs(300)))
            .max_idle_per_host(10000)
            .build::<_, Body>(https)
    }
}
