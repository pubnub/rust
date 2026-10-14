//! Access token parser module.
//!
//! This module contains the [`parse_token`] function, which produces a
//! [`Token`] with information about the permissions granted to the token.

use crate::core::Deserializer;
use crate::{
    core::utils::permission_bits as permissions,
    core::PubNubError,
    lib::{
        alloc::{
            format,
            string::{String, ToString},
        },
        collections::HashMap,
    },
};
use base64::{engine::general_purpose, Engine};
#[cfg(feature = "serde")]
use ciborium::de::from_reader;

#[cfg(feature = "serde")]
struct CiboriumDeserializer;

#[cfg(feature = "serde")]
impl Deserializer for CiboriumDeserializer {
    fn deserialize<Token: for<'de> serde::Deserialize<'de>>(
        &self,
        bytes: &[u8],
    ) -> Result<Token, PubNubError> {
        from_reader(bytes).map_err(|e| PubNubError::TokenDeserialization {
            details: e.to_string(),
        })
    }
}

/// The [`parse_token`] function decodes an existing token and returns the
/// struct containing permissions embedded in that token.
/// The client can use this method for debugging to check the permissions to the
/// resources.
#[cfg(feature = "serde")]
pub fn parse_token(token: &str) -> Result<Token, PubNubError> {
    parse_token_with(token, CiboriumDeserializer)
}

/// The [`parse_token`] function decodes an existing token and returns the
/// struct deserialized by provided cbor deserializer containing permissions
/// embedded in that token. The client can use this method for debugging to
/// check the permissions to the resources.
pub fn parse_token_with<D>(token: &str, deserializer: D) -> Result<Token, PubNubError>
where
    D: Deserializer,
{
    let token_bytes = general_purpose::URL_SAFE
        .decode(format!("{token}{}", "=".repeat(token.len() % 4)).as_bytes())
        .map_err(|e| PubNubError::TokenDeserialization {
            details: e.to_string(),
        })?;

    deserializer.deserialize(&token_bytes)
}

/// Version based access token.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Token {
    /// Decoded token information (version 2).
    V2(TokenV2),
}

/// Access token (version 2) with information about resources and their
/// permissions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TokenV2 {
    /// Access token version (version 2).
    #[cfg_attr(feature = "serde", serde(rename = "v"))]
    pub version: u8,

    /// Unix-timestamp with the date when token has been granted.
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    pub timestamp: u32,

    /// Duration (in minutes) during which this token permissions are valid.
    pub ttl: u32,

    /// Dedicated user ID which can use this access token (if provided during
    /// grant token call).
    #[cfg_attr(feature = "serde", serde(rename = "uuid"))]
    pub authorized_user_id: Option<String>,

    /// Permissions for resources identified by their names.
    #[cfg_attr(feature = "serde", serde(rename = "res"))]
    pub resources: Option<TokenResources>,

    /// Permissions for resources identified by regular expressions.
    #[cfg_attr(feature = "serde", serde(rename = "pat"))]
    pub patterns: Option<TokenResources>,

    /// Permissions for enumerate-able resource categories.
    #[cfg_attr(feature = "serde", serde(rename = "cat"))]
    pub categories: Option<TokenCategories>,

    /// Extra metadata to which has been included into access token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub meta: HashMap<String, MetaValue>,
}

/// Typed resource permissions map.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TokenResources {
    /// `Channel`-based endpoints permission map between channel name / regexp
    /// and set of permissions.
    #[cfg_attr(feature = "serde", serde(rename = "chan", default))]
    pub channels: HashMap<String, ResourcePermissions>,

    /// `Channel group`-based endpoints permission map between channel
    /// name / regexp and set of permissions.
    #[cfg_attr(feature = "serde", serde(rename = "grp", default))]
    pub groups: HashMap<String, ResourcePermissions>,

    /// `UserId`-based endpoints permission map between channel
    /// name / regexp and set of permissions.
    #[cfg_attr(feature = "serde", serde(rename = "uuid", default))]
    pub users: HashMap<String, ResourcePermissions>,
}

/// Typed category permissions map.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TokenCategories {
    /// `Channels` enumeration endpoint permission.
    #[cfg_attr(feature = "serde", serde(rename = "chan", default))]
    pub channels: Option<CategoryPermissions>,

    /// `Uuids` enumeration endpoint permission.
    #[cfg_attr(feature = "serde", serde(rename = "uuid", default))]
    pub uuids: Option<CategoryPermissions>,
}

impl From<u8> for ResourcePermissions {
    fn from(int: u8) -> Self {
        ResourcePermissions {
            read: int & permissions::READ != 0,
            write: int & permissions::WRITE != 0,
            manage: int & permissions::MANAGE != 0,
            delete: int & permissions::DELETE != 0,
            create: int & permissions::CREATE != 0,
            get: int & permissions::GET != 0,
            update: int & permissions::UPDATE != 0,
            join: int & permissions::JOIN != 0,
        }
    }
}

/// Resource permissions map.
///
/// This structure contains information about permissions which has been granted
/// to specific resource.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "serde", serde(from = "u8"))]
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

impl From<u8> for CategoryPermissions {
    fn from(int: u8) -> Self {
        CategoryPermissions {
            get: int & permissions::GET != 0,
        }
    }
}

/// Category permissions map.
///
/// This structure contains information about permissions which has been granted
/// to specific category.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "serde", serde(from = "u8"))]
pub struct CategoryPermissions {
    /// Whether or not the category has **get** permission.
    pub get: bool,
}

/// Enum for values associated with token.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
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
    use crate::{
        dx::parse_token::MetaValue::{Float, Integer, Null, String},
        lib::core::ops::Deref,
    };

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
                patterns: Some(TokenResources {
                    channels: [("channel.*".into(), 2.into())].into(),
                    groups: [("channelGroup.*".into(), 4.into())].into(),
                    users: [("uuid.*".into(), 104.into())].into(),
                }),
                resources: Some(TokenResources {
                    users: [].into(),
                    groups: [("channelGroup".into(), 1.into())].into(),
                    channels: [("channelResource".into(), 255.into())].into(),
                }),
                categories: None,
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

    #[test]
    fn parse_token_with_categories_only() {
        let Token::V2(token) = parse_token(
            "qGF2AmF0GmFLd-NjdHRsGQWgY3Jlc6BjcGF0oGRtZXRhoGNjYXSiZGNoYW4YIGR1dWlkGCBjc2lnRAECAwQ=",
        )
        .unwrap();
        assert_eq!(token.version, 2);
        assert_eq!(token.timestamp, 1632335843);
        assert_eq!(token.ttl, 1440);
        let categories = token.categories.expect("categories should be present");
        assert_eq!(categories.channels, Some(CategoryPermissions { get: true }));
        assert_eq!(categories.uuids, Some(CategoryPermissions { get: true }));
    }

    #[test]
    fn deserialize_categories_without_uuids() {
        // CBOR for {"chan": 32}
        let cbor = [0xa1, 0x64, b'c', b'h', b'a', b'n', 0x18, 0x20];
        let categories: TokenCategories = ciborium::de::from_reader(&cbor[..]).unwrap();
        assert_eq!(categories.channels, Some(CategoryPermissions { get: true }));
        assert_eq!(categories.uuids, None);
    }

    #[test]
    fn deserialize_categories_without_channels() {
        // CBOR for {"uuid": 32}
        let cbor = [0xa1, 0x64, b'u', b'u', b'i', b'd', 0x18, 0x20];
        let categories: TokenCategories = ciborium::de::from_reader(&cbor[..]).unwrap();
        assert_eq!(categories.channels, None);
        assert_eq!(categories.uuids, Some(CategoryPermissions { get: true }));
    }

    #[test]
    fn ignore_unknown_category_keys() {
        // CBOR for {"chan": 32, "foo": 1}
        let cbor = [
            0xa2, 0x64, b'c', b'h', b'a', b'n', 0x18, 0x20, 0x63, b'f', b'o', b'o', 0x01,
        ];
        let categories: TokenCategories = ciborium::de::from_reader(&cbor[..]).unwrap();
        assert_eq!(categories.channels, Some(CategoryPermissions { get: true }));
    }

    #[test]
    fn deserialize_empty_resources() {
        // CBOR for {}
        let resources: TokenResources = ciborium::de::from_reader(&[0xa0u8][..]).unwrap();
        assert!(resources.channels.is_empty());
        assert!(resources.groups.is_empty());
        assert!(resources.users.is_empty());
    }
}
