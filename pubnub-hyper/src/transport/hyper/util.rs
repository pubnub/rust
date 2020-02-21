//! Common utilities.

use super::error;
use crate::core::json;
use futures_util::stream::StreamExt;
use hyper::{Body, Response, Uri};
use log::debug;

use super::Hyper;

pub(super) fn build_uri(hyper: &Hyper, path_and_query: &str) -> Result<Uri, http::Error> {
    let url = Uri::builder()
        .scheme("https")
        .authority(hyper.origin.as_str())
        .path_and_query(path_and_query)
        .build()?;
    debug!("URL: {}", url);
    Ok(url)
}

pub(super) async fn handle_json_response(
    response: Response<Body>,
) -> Result<json::JsonValue, error::Error> {
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
