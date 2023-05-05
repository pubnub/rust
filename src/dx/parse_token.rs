//! Access token parser module.
//!
//! This module contains the [`parse_token`] function, which produces a
//! [`Token`] with information about the permissions granted to the token.

use crate::core::PubNubError;
use base64::{engine::general_purpose, Engine};
use ciborium::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, ops::Deref};

/// The [`parse_token`] function decodes an existing token and returns the
/// struct containing permissions embedded in that token.
/// The client can use this method for debugging to check the permissions to the
/// resources.
pub fn parse_token(token: &str) -> Result<Token, PubNubError> {
    let token_bytes = general_purpose::URL_SAFE
        .decode(format!("{token}{}", "=".repeat(token.len() % 4)).as_bytes())
        .map_err(|e| PubNubError::TokenDeserialization {
            details: e.to_string(),
        })?;

    from_reader(token_bytes.deref()).map_err(|e| PubNubError::TokenDeserialization {
        details: e.to_string(),
    })
}

/// Version based access token.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Token {
    /// Decoded token information (version 2).
    V2(TokenV2),
}

/// Access token (version 2) with information about resources and their
/// permissions.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TokenV2 {
    /// Access token version (version 2).
    #[serde(rename = "v")]
    pub version: u8,

    /// Unix-timestamp with the date when token has been granted.
    #[serde(rename = "t")]
    pub timestamp: u32,

    /// Duration (in minutes) during which this token permissions are valid.
    #[serde(rename = "ttl")]
    pub ttl: u32,

    /// Dedicated user ID which can use this access token (if provided during
    /// grant token call).
    #[serde(rename = "uuid")]
    pub authorized_user_id: Option<String>,

    /// Permissions for resources identified by their names.
    #[serde(rename = "res")]
    pub resources: TokenResources,

    /// Permissions for resources identified by regular expressions.
    #[serde(rename = "pat")]
    pub patterns: TokenResources,

    /// Extra metadata to which has been included into access token.
    pub meta: HashMap<String, MetaValue>,
}

/// Typed resource permissions map.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TokenResources {
    /// `Channel`-based endpoints permission map between channel name / regexp
    /// and set of permissions.
    #[serde(rename = "chan")]
    pub channels: HashMap<String, ResourcePermissions>,

    /// `Channel group`-based endpoints permission map between channel
    /// name / regexp and set of permissions.
    #[serde(rename = "grp")]
    pub groups: HashMap<String, ResourcePermissions>,

    /// `UserId`-based endpoints permission map between channel
    /// name / regexp and set of permissions.
    #[serde(rename = "uuid")]
    pub users: HashMap<String, ResourcePermissions>,
}

impl From<u8> for ResourcePermissions {
    fn from(int: u8) -> Self {
        ResourcePermissions {
            read: int & 1 != 0,
            write: int & 2 != 0,
            manage: int & 4 != 0,
            delete: int & 8 != 0,
            create: int & 16 != 0,
            get: int & 32 != 0,
            update: int & 64 != 0,
            join: int & 128 != 0,
        }
    }
}

/// Resource permissions map.
///
/// This structure contains information about permissions which has been granted
/// to specific resource.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[allow(dead_code)]
#[serde(from = "u8")]
pub struct ResourcePermissions {
    /// Whether or not the resource has **read** permission.
    pub read: bool,

    /// Whether or not the resource has **write** permission.
    pub write: bool,

    /// Whether or not the resource has **manage** permission.
    pub manage: bool,

    /// Whether or not the resource has **delete** permission.
    pub delete: bool,

    /// Whether or not the resource has **create** permission.
    pub create: bool,

    /// Whether or not the resource has **get** permission.
    pub get: bool,

    /// Whether or not the resource has **update** permission.
    pub update: bool,

    /// Whether or not the resource has **join** permission.
    pub join: bool,
}

/// Enum for values associated with token.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    /// `String` value.
    String(String),

    /// `Integer` value.
    Integer(i64),

    /// `Float` / `double` value.
    Float(f64),

    /// `Boolean` value.
    Bool(bool),

    /// `null` value.
    Null,
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::dx::parse_token::MetaValue::{Float, Integer, Null, String};

    impl PartialEq for MetaValue {
        fn eq(&self, other: &Self) -> bool {
            use MetaValue::*;

            match (self.clone(), other.clone()) {
                (String(v1), String(v2)) => v1.deref() == v2.deref(),
                (Integer(v1), Integer(v2)) => v1 == v2,
                (Float(v1), Float(v2)) => (v1 - v2).abs() < 0.001,
                (Bool(v1), Bool(v2)) => v1 == v2,
                (Null, Null) => true,
                _ => false,
            }
        }
    }

    impl Eq for MetaValue {}

    #[test]
    fn test_parse_token() {
        let base64_token = "qEF2AkF0GmQ1YSpDdHRsGQU5Q3Jlc6VEY2hhbqFvY2hhbm5lbFJlc291cmNlGP9DZ3JwoWxjaGFubmVsR3JvdXABQ3NwY6BDdXNyoER1dWlkoENwYXSlRGNoYW6haWNoYW5uZWwuKgJDZ3JwoW5jaGFubmVsR3JvdXAuKgRDc3BjoEN1c3KgRHV1aWShZnV1aWQuKhhoRG1ldGGkZG1ldGFkZGF0YWdpbnRlZ2VyGQU5ZW90aGVy9mVmbG9hdPtAKr1wo9cKPUR1dWlkZHV1aWRDc2lnWCAbOhXPSWx05l4c3Iuf-SWVOVpLM6xyto3lVPdMKdhJ2A";
        let token = parse_token(base64_token).unwrap();
        assert_eq!(
            Token::V2(TokenV2 {
                version: 2,
                ttl: 1337,
                timestamp: 1681219882,
                patterns: TokenResources {
                    channels: [("channel.*".into(), 2.into())].into(),
                    groups: [("channelGroup.*".into(), 4.into())].into(),
                    users: [("uuid.*".into(), 104.into())].into(),
                },
                resources: TokenResources {
                    users: [].into(),
                    groups: [("channelGroup".into(), 1.into())].into(),
                    channels: [("channelResource".into(), 255.into())].into(),
                },
                authorized_user_id: Some("uuid".into()),
                meta: HashMap::from([
                    ("meta".into(), String("data".into())),
                    ("other".into(), Null),
                    ("integer".into(), Integer(1337)),
                    ("float".into(), Float(13.37))
                ])
            }),
            token
        );
    }
}
