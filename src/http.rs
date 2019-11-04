use crate::error::Error;
use crate::message::{Message, Timetoken, Type};
use hyper::{client::HttpConnector, Body, Client, Uri};
pub(crate) use hyper_tls::HttpsConnector;

pub(crate) type HttpClient = Client<HttpsConnector<HttpConnector>, Body>;

/// # Send a publish request and return the JSON response
pub(crate) async fn publish_request(
    http_client: &HttpClient,
    url: Uri,
) -> Result<Timetoken, Error> {
    // Send network request
    let res = http_client.get(url).await;
    let mut body = res.unwrap().into_body();
    let mut bytes = Vec::new();

    // Receive the response as a byte stream
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk?);
    }

    // Convert the resolved byte stream to JSON
    let data = std::str::from_utf8(&bytes)?;
    let data_json = json::parse(data)?;
    let timetoken = Timetoken {
        t: data_json[2].to_string(),
        r: 0, // TODO
    };

    // Deliever the timetoken response from PubNub
    Ok(timetoken)
}

/// # Send a subscribe request and return the JSON messages received
pub(crate) async fn subscribe_request(
    http_client: &HttpClient,
    url: Uri,
) -> Result<(Vec<Message>, Timetoken), Error> {
    // Send network request
    let res = http_client.get(url).await;
    let mut body = res.unwrap().into_body();
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
        t: data_json["t"]["t"].to_string(),
        r: data_json["t"]["r"].as_u32().unwrap_or(0),
    };

    // Capture Messages in Vec Buffer
    let messages = data_json["m"]
        .members()
        .map(|message| Message {
            message_type: Type::from_json(&message["e"]),
            route: message["b"].as_str().map(ToString::to_string),
            channel: message["c"].to_string(),
            json: message["d"].clone(),
            metadata: message["u"].clone(),
            timetoken: Timetoken {
                t: message["p"]["t"].to_string(),
                r: message["p"]["r"].as_u32().unwrap_or(0),
            },
            client: message["i"].as_str().map(ToString::to_string),
            subscribe_key: message["k"].to_string(),
            flags: message["f"].as_u32().unwrap_or(0),
        })
        .collect::<Vec<_>>();

    // Deliver the message response from PubNub
    Ok((messages, timetoken))
}
