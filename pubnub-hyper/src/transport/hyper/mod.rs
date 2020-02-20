//! Hyper transport implementation.
use crate::core::data::{
    message::{Message, Type},
    request,
    timetoken::Timetoken,
};
use crate::core::json;
use crate::core::Transport;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use pubnub_util::encoded_channels_list::EncodedChannelsList;

use async_trait::async_trait;

use derive_builder::Builder;
use futures_util::stream::StreamExt;
use hyper::{client::HttpConnector, Body, Client, Response, Uri};
use hyper_tls::HttpsConnector;
use std::time::Duration;

pub mod error;

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

macro_rules! encode_json {
    ($value:expr => $to:ident) => {
        let value_string = json::stringify($value);
        let $to = {
            use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
            utf8_percent_encode(&value_string, NON_ALPHANUMERIC)
        };
    };
}

#[async_trait]
impl Transport for Hyper {
    type Error = error::Error;

    async fn publish_request(&self, request: request::Publish) -> Result<Timetoken, Self::Error> {
        // Prepare encoded message and channel.
        encode_json!(request.payload => encoded_payload);
        let encoded_channel = utf8_percent_encode(&request.channel, NON_ALPHANUMERIC);

        // Prepare the URL.
        let path_and_query = format!(
            "/publish/{pub_key}/{sub_key}/0/{channel}/0/{message}",
            pub_key = self.publish_key,
            sub_key = self.subscribe_key,
            channel = encoded_channel,
            message = encoded_payload,
        );
        let url = self.build_uri(&path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_json_response(response).await?;

        // Parse timetoken.
        let timetoken = Timetoken {
            t: data_json[2].as_str().unwrap().parse().unwrap(),
            r: 0, // TODO
        };

        Ok(timetoken)
    }

    async fn subscribe_request(
        &self,
        request: request::Subscribe,
    ) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        // TODO: add caching of repeating params to avoid reencoding.

        // Prepare encoded channels and channel_groups.
        let encoded_channels = EncodedChannelsList::from(request.channels);
        let encoded_channel_groups = EncodedChannelsList::from(request.channel_groups);

        // Prepare the URL.
        let path_and_query = format!(
            "/v2/subscribe/{sub_key}/{channels}/0?channel-group={channel_groups}&tt={tt}&tr={tr}",
            sub_key = self.subscribe_key,
            channels = encoded_channels,
            channel_groups = encoded_channel_groups,
            tt = request.timetoken.t,
            tr = request.timetoken.r,
        );
        let url = self.build_uri(&path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_json_response(response).await?;

        // Parse timetoken.
        let timetoken = Timetoken {
            t: data_json["t"]["t"].as_str().unwrap().parse().unwrap(),
            r: data_json["t"]["r"].as_u32().unwrap_or(0),
        };

        // Parse messages.
        let messages = data_json["m"]
            .members()
            .map(|message| Message {
                message_type: Type::from_json(&message["e"]),
                route: message["b"].as_str().map(str::to_string),
                channel: message["c"].to_string(),
                json: message["d"].clone(),
                metadata: message["u"].clone(),
                timetoken: Timetoken {
                    t: message["p"]["t"].as_str().unwrap().parse().unwrap(),
                    r: message["p"]["r"].as_u32().unwrap_or(0),
                },
                client: message["i"].as_str().map(str::to_string),
                subscribe_key: message["k"].to_string(),
                flags: message["f"].as_u32().unwrap_or(0),
            })
            .collect::<Vec<_>>();

        Ok((messages, timetoken))
    }
}

async fn handle_json_response(response: Response<Body>) -> Result<json::JsonValue, error::Error> {
    let mut body = response.into_body();
    let mut bytes = Vec::new();

    // Receive the response as a byte stream
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk?);
    }

    // Convert the resolved byte stream to JSON.
    let data = std::str::from_utf8(&bytes)?;
    let data_json = json::parse(data)?;

    Ok(data_json)
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
