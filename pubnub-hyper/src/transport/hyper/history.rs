//! History.

use super::util::{build_uri, handle_json_response, json_as_array, json_as_object};
use super::{error, Hyper};
use crate::core::data::{request, response};
use crate::core::json;
use crate::core::TransportService;
use async_trait::async_trait;
use http::{Method, Request};
use hyper::{Body, Response};
use pubnub_core::data::{channel, history};
use pubnub_util::uritemplate::UriTemplate;
use std::collections::HashMap;

async fn handle_history_response(
    response: Response<Body>,
) -> Result<json::JsonValue, error::Error> {
    let history_data = handle_json_response(response).await?;

    if history_data["error"] == true {
        let error_message = history_data["message"].to_string();
        return Err(error::Error::Server(error_message));
    }

    Ok(history_data)
}

#[async_trait]
impl TransportService<request::GetHistory> for Hyper {
    type Response = response::GetHistory;
    type Error = error::Error;

    async fn call(&self, request: request::GetHistory) -> Result<Self::Response, Self::Error> {
        let request::GetHistory {
            channels,
            max,
            reverse,
            start,
            end,
            include_metadata,
        } = request;

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v3/history/sub-key/{sub_key}/channel/{channels}{?max,reverse,start,end,include_meta}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list("channels", channels)
        .set_optional_scalar("max", max)
        .set_optional_scalar("reverse", reverse)
        .set_optional_scalar("start", start)
        .set_optional_scalar("end", end)
        .set_optional_scalar("include_meta", include_metadata)
        .build();
        let url = build_uri(self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_history_response(response).await?;

        // Parse response.
        let channels = parse_get_history(&data_json)
            .ok_or(error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(channels)
    }
}

#[async_trait]
impl TransportService<request::DeleteHistory> for Hyper {
    type Response = response::DeleteHistory;
    type Error = error::Error;

    async fn call(&self, request: request::DeleteHistory) -> Result<Self::Response, Self::Error> {
        let request::DeleteHistory {
            channels,
            start,
            end,
        } = request;

        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v3/history/sub-key/{sub_key}/channel/{channels}{?start,end}")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .set_list("channels", channels)
                .set_optional_scalar("start", start)
                .set_optional_scalar("end", end)
                .build();
        let url = build_uri(self, &path_and_query)?;

        // Prepare the request.
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(url)
            .body(Body::empty())?;

        // Send network request.
        let response = self.http_client.request(req).await?;
        let _data_json = handle_history_response(response).await?;

        Ok(())
    }
}

#[async_trait]
impl TransportService<request::MessageCountsWithTimetoken> for Hyper {
    type Response = response::MessageCountsWithTimetoken;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::MessageCountsWithTimetoken,
    ) -> Result<Self::Response, Self::Error> {
        let request::MessageCountsWithTimetoken {
            channels,
            timetoken,
        } = request;

        // Prepare the URL.
        let path_and_query =
            UriTemplate::new("/v3/history/sub-key/{sub_key}/message-counts/{channels}{?timetoken}")
                .set_scalar("sub_key", self.subscribe_key.clone())
                .set_list("channels", channels)
                .set_scalar("timetoken", timetoken)
                .build();
        let url = build_uri(self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_history_response(response).await?;

        // Parse response.
        let channels = parse_message_counts(&data_json)
            .ok_or(error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(channels)
    }
}

#[async_trait]
impl TransportService<request::MessageCountsWithChannelTimetokens> for Hyper {
    type Response = response::MessageCountsWithChannelTimetokens;
    type Error = error::Error;

    async fn call(
        &self,
        request: request::MessageCountsWithChannelTimetokens,
    ) -> Result<Self::Response, Self::Error> {
        let request::MessageCountsWithChannelTimetokens { channels } = request;

        // Unzip Vector-of-Tuples into separate Vectors.
        let (names, timetokens): (Vec<_>, Vec<_>) = channels.into_iter().unzip();

        // Prepare the URL.
        let path_and_query = UriTemplate::new(
            "/v3/history/sub-key/{sub_key}/message-counts/{channels}{?channelsTimetoken}",
        )
        .set_scalar("sub_key", self.subscribe_key.clone())
        .set_list("channels", names)
        .set_list("channelsTimetoken", timetokens)
        .build();
        let url = build_uri(self, &path_and_query)?;

        // Send network request.
        let response = self.http_client.get(url).await?;
        let data_json = handle_history_response(response).await?;

        // Parse response.
        let channels = parse_message_counts(&data_json)
            .ok_or(error::Error::UnexpectedResponseSchema(data_json))?;
        Ok(channels)
    }
}

fn parse_item(item: &json::object::Object) -> Option<history::Item> {
    let message = item["message"].clone();
    let timetoken = item["timetoken"].as_str()?.parse().ok()?;
    let metadata = item["meta"].clone();
    Some(history::Item {
        message,
        timetoken,
        metadata,
    })
}

fn parse_get_history(
    data_json: &json::JsonValue,
) -> Option<HashMap<channel::Name, Vec<history::Item>>> {
    let channels = json_as_object(&data_json["channels"])?;
    channels
        .iter()
        .map(|(channel, items)| {
            let items = json_as_array(items)?;
            let items: Option<Vec<_>> = items
                .iter()
                .map(|item| match json_as_object(item) {
                    Some(item) => parse_item(item),
                    None => None,
                })
                .collect();
            Some((channel.parse().ok()?, items?))
        })
        .collect()
}

fn parse_message_counts(data_json: &json::JsonValue) -> Option<HashMap<channel::Name, usize>> {
    let channels = json_as_object(&data_json["channels"])?;
    channels
        .iter()
        .map(|(key, val)| Some((key.parse().ok()?, val.as_usize()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        history,
        json::{self, JsonValue},
        parse_item,
    };

    #[test]
    fn test_parse_item() {
        let sample = json::object! {
            "message": { "my_payload": "my_value" },
            "timetoken": "15909263655404500"
        };
        let sample_object = match sample {
            JsonValue::Object(val) => val,
            _ => panic!("invald test"),
        };

        let item = parse_item(&sample_object).unwrap();

        let expected_item = history::Item {
            message: json::object! { "my_payload": "my_value" },
            timetoken: 15_909_263_655_404_500,
            metadata: json::Null,
        };

        assert_eq!(item, expected_item);
    }
}
