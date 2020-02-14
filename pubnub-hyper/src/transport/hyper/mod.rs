use crate::core::json;
use crate::core::Transport;
use crate::core::{Instance, Message, Timetoken, Type};

use async_trait::async_trait;

use futures_util::stream::StreamExt;
use hyper::{client::HttpConnector, Body, Client, Uri};
use hyper_tls::HttpsConnector;
use std::sync::Arc;
use std::time::Duration;

pub mod error;

type HttpClient = Client<HttpsConnector<HttpConnector>>;

/// Implements transport for PubNub using the `hyper` crate to communicate with
/// the PubNub REST API.
#[derive(Debug, Clone)]
pub struct Hyper {
    pub http_client: HttpClient,
}

#[async_trait]
impl Transport for Hyper {
    type Error = error::Error;

    async fn publish_request(&self, url: Uri) -> Result<Timetoken, Self::Error> {
        // Send network request
        let res = self.http_client.get(url).await?;
        let mut body = res.into_body();
        let mut bytes = Vec::new();

        // Receive the response as a byte stream
        while let Some(chunk) = body.next().await {
            bytes.extend(chunk?);
        }

        // Convert the resolved byte stream to JSON
        let data = std::str::from_utf8(&bytes)?;
        let data_json = json::parse(data)?;
        let timetoken = Timetoken {
            t: data_json[2].as_str().unwrap().parse().unwrap(),
            r: 0, // TODO
        };

        // Deliever the timetoken response from PubNub
        Ok(timetoken)
    }

    async fn subscribe_request(&self, url: Uri) -> Result<(Vec<Message>, Timetoken), Self::Error> {
        // Send network request
        let res = self.http_client.get(url).await?;
        let mut body = res.into_body();
        let mut bytes = Vec::new();

        // Receive the response as a byte stream
        while let Some(chunk) = body.next().await {
            bytes.extend(chunk?);
        }

        // Convert the resolved byte stream to JSON
        let data = std::str::from_utf8(&bytes)?;
        let data_json = json::parse(data)?;

        // Decode the stream timetoken
        let timetoken = Timetoken {
            t: data_json["t"]["t"].as_str().unwrap().parse().unwrap(),
            r: data_json["t"]["r"].as_u32().unwrap_or(0),
        };

        // Capture Messages in Vec Buffer
        let messages = data_json["m"]
            .members()
            .map(|message| {
                Arc::new(Instance {
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
            })
            .collect::<Vec<_>>();

        // Deliver the message response from PubNub
        Ok((messages, timetoken))
    }
}

impl Default for Hyper {
    #[must_use]
    fn default() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .keep_alive_timeout(Some(Duration::from_secs(300)))
            .max_idle_per_host(10000)
            .build::<_, Body>(https);
        Self {
            http_client: client,
        }
    }
}
