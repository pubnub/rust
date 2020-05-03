//! PAMv3.

use super::util::{build_uri, handle_json_response, json_as_object};
use super::{error, Hyper};
use crate::core::data::{pam, request, response};
use crate::core::json;
use crate::core::TransportService;
use async_trait::async_trait;
use hyper::{Body, Method, Request, Response, StatusCode};
use pubnub_util::pam_signature;
use pubnub_util::uritemplate::UriTemplate;
use std::collections::HashMap;

#[async_trait]
impl TransportService<request::Grant> for Hyper {
    type Response = response::Grant;
    type Error = error::Error;

    async fn call(&self, request: request::Grant) -> Result<Self::Response, Self::Error> {
        // Abort if we don't have a secret key.
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| error::ConfigurationError::SecretKeyUnavailable)?;

        // Prepare the request body and the signature.
        let body = prepare_grant_body(request);
        let timestamp = get_unix_time();
        let signature = prepare_signature(
            secret_key,
            &self.subscribe_key,
            &self.publish_key,
            timestamp,
            body.as_str(),
        );

        // Prepare the URL.
        let path_and_query = UriTemplate::new("/v3/pam/{sub_key}/grant{?signature,timestamp}")
            .set_scalar("sub_key", self.subscribe_key.clone())
            .set_scalar("signature", signature)
            .set_scalar("timestamp", timestamp.to_string())
            .build();
        let url = build_uri(&self, &path_and_query)?;

        // Prepare the request.
        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("content-type", "application/json")
            .body(Body::from(body))?;

        // Send network request.
        let response = self.http_client.request(req).await?;
        handle_grant_response(response).await
    }
}

fn prepare_grant_body(input: pam::GrantBody) -> String {
    let map = |input: &HashMap<String, pam::BitMask>| -> json::JsonValue {
        let mut data = json::JsonValue::new_object();
        for (key, val) in input {
            data[key] = val.0.into();
        }
        data
    };
    let resources = {
        let input = &input.permissions.resources;
        let mut data = json::JsonValue::new_object();
        data["channels"] = map(&input.channels);
        data["groups"] = map(&input.groups);
        data["users"] = map(&input.users);
        data["spaces"] = map(&input.spaces);
        data
    };
    let patterns = {
        let input = &input.permissions.patterns;
        let mut data = json::JsonValue::new_object();
        data["channels"] = map(&input.channels);
        data["groups"] = map(&input.groups);
        data["users"] = map(&input.users);
        data["spaces"] = map(&input.spaces);
        data
    };
    let permissions = {
        let mut data = json::JsonValue::new_object();
        data["resources"] = resources;
        data["patterns"] = patterns;
        data["meta"] = input.permissions.meta;
        data
    };
    let grant_body = {
        let mut data = json::JsonValue::new_object();
        data["ttl"] = input.ttl.into();
        data["permissions"] = permissions;
        data
    };
    json::stringify(grant_body)
}

/// Obtain UNIX timestamp.
fn get_unix_time() -> u64 {
    let current = std::time::SystemTime::now();
    let since_the_epoch = current
        .duration_since(std::time::UNIX_EPOCH)
        .expect("things seem to be happening before the unix epoch, check system clock");
    since_the_epoch.as_secs()
}

/// Prepare the signature.
fn prepare_signature(
    secret_key: &str,
    subscribe_key: &str,
    publish_key: &str,
    timestamp: u64,
    body: &str,
) -> String {
    pam_signature::sign(
        secret_key,
        pam_signature::Request {
            publish_key,
            method: "POST",
            path: &format!("/v3/pam/{}/grant", subscribe_key),
            query: &format!("timestamp={}", timestamp),
            body,
        },
    )
}

async fn handle_grant_response(response: Response<Body>) -> Result<response::Grant, error::Error> {
    match response.status() {
        StatusCode::OK => {
            let data_json = handle_json_response(response).await?;
            let err_fn = || error::Error::UnexpectedResponseSchema(data_json.clone());
            let token = {
                let data = json_as_object(&data_json["data"]).ok_or_else(err_fn)?;
                let token = data["token"].as_str().ok_or_else(err_fn)?;
                token.to_owned()
            };
            Ok(token)
        }
        StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
            let data = handle_json_response(response).await?;
            let error_message: String = format!("{}", data["error"]["message"]);
            Err(error::Error::Server(error_message))
        }
        _ => Err(error::Error::Server(format!(
            "Server responded with an unexpected status code: {}",
            response.status()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::pam;
    use super::prepare_grant_body;
    use super::prepare_signature;
    use std::collections::HashMap;

    #[test]
    fn test_prepare_grant_body() {
        let sample = pam::GrantBody {
            ttl: 10,
            permissions: pam::Permissions {
                resources: pam::Resources {
                    channels: {
                        let mut map = HashMap::new();
                        map.insert("channel_a".into(), pam::BitMask(4));
                        map.insert("channel_b".into(), pam::BitMask(0));
                        map
                    },
                    groups: {
                        let mut map = HashMap::new();
                        map.insert("groups_a".into(), pam::BitMask(4));
                        map.insert("groups_b".into(), pam::BitMask(0));
                        map
                    },
                    users: {
                        let mut map = HashMap::new();
                        map.insert("users_a".into(), pam::BitMask(4));
                        map.insert("users_b".into(), pam::BitMask(0));
                        map
                    },
                    spaces: {
                        let mut map = HashMap::new();
                        map.insert("spaces_a".into(), pam::BitMask(4));
                        map.insert("spaces_b".into(), pam::BitMask(0));
                        map
                    },
                },
                patterns: pam::Patterns {
                    channels: {
                        let mut map = HashMap::new();
                        map.insert("channel_c".into(), pam::BitMask(4));
                        map.insert("channel_d".into(), pam::BitMask(0));
                        map
                    },
                    groups: {
                        let mut map = HashMap::new();
                        map.insert("groups_c".into(), pam::BitMask(4));
                        map.insert("groups_d".into(), pam::BitMask(0));
                        map
                    },
                    users: {
                        let mut map = HashMap::new();
                        map.insert("users_c".into(), pam::BitMask(4));
                        map.insert("users_d".into(), pam::BitMask(0));
                        map
                    },
                    spaces: {
                        let mut map = HashMap::new();
                        map.insert("spaces_c".into(), pam::BitMask(4));
                        map.insert("spaces_d".into(), pam::BitMask(0));
                        map
                    },
                },
                meta: json::object! {
                    "user_id" => "qwerty",
                },
            },
        };

        let body = prepare_grant_body(sample);

        assert_eq!(
            json::parse(&body).unwrap(),
            json::object! {
              "ttl": 10,
              "permissions": {
                "resources": {
                  "channels": {
                    "channel_b": 0,
                    "channel_a": 4
                  },
                  "groups": {
                    "groups_a": 4,
                    "groups_b": 0
                  },
                  "users": {
                    "users_a": 4,
                    "users_b": 0
                  },
                  "spaces": {
                    "spaces_b": 0,
                    "spaces_a": 4
                  }
                },
                "patterns": {
                  "channels": {
                    "channel_c": 4,
                    "channel_d": 0
                  },
                  "groups": {
                    "groups_c": 4,
                    "groups_d": 0
                  },
                  "users": {
                    "users_c": 4,
                    "users_d": 0
                  },
                  "spaces": {
                    "spaces_c": 4,
                    "spaces_d": 0
                  }
                },
                "meta": {
                  "user_id": "qwerty"
                }
              }
            }
        );
    }

    fn test_prepare_signature() {
        prepare_signature(
            "wMfbo9G0xVUG8yfTfYw5qIdfJkTd7A",
            "demo",
            "demo",
            123456789,
            "{}",
        )
    }
}
