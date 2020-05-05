//! Hyper transport implementation.

use crate::core::data::uuid::UUID;
use crate::core::Transport;
use derive_builder::Builder;
use getset::Getters;
use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use std::time::Duration;

pub mod error;
pub mod pam;
pub mod presence;
pub mod pubsub;

#[macro_use]
pub(crate) mod util;

type HttpClient = Client<HttpsConnector<HttpConnector>>;

/// Implements transport for PubNub using the `hyper` crate to communicate with
/// the PubNub REST API.
#[derive(Debug, Clone, Builder, Getters)]
#[getset(get = "pub")]
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
    /// Secret key matching the subscribe key.
    #[builder(setter(into, strip_option), default = "None")]
    secret_key: Option<String>,

    /// The authority URL part to use to connet to the PubNub edge network
    #[builder(setter(into), default = "\"ps.pndsn.com\".to_owned()")]
    origin: String,
    /// User-Agent header value to use at HTTP requests.
    #[builder(setter(into), default = "\"Rust-Agent\".to_owned()")]
    agent: String,

    /// A UUID to identify as.
    #[builder(setter(into), default = "Self::default_uuid()")]
    uuid: UUID,
}

impl Hyper {
    /// Produces a builder that can be used to construct [`Hyper`] transport.
    #[must_use]
    #[allow(clippy::new_ret_no_self)] // builder pattern should be detected
    pub fn new() -> HyperBuilder {
        HyperBuilder::default()
    }
}

impl Transport for Hyper {
    type Error = error::Error;
}

impl HyperBuilder {
    fn default_http_client() -> HttpClient {
        let https = HttpsConnector::new();
        Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .pool_max_idle_per_host(10000)
            .build::<_, Body>(https)
    }

    fn default_uuid() -> UUID {
        UUID::random()
    }
}
