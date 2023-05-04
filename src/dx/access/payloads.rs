//! Request payloads module.
//!
//! This module contains: [`Permission`] struct and it's implementation of
//! [`ChannelPermission`],  [`ChannelGroupPermission`] and [`UserIdPermission`]
//! traits.

use crate::core::Serializer;
use crate::lib::{
    a::boxed::Box,
    a::string::{String, ToString},
};
use crate::{
    core::Deserializer,
    dx::access::{permissions::*, types::MetaValue, GrantTokenRequest, GrantTokenResponseBody},
};
use hashbrown::HashMap;

/// Resource and pattern-based permissions payload.
///
/// This type used by [`GrantTokenPayload`] to store permissions for specific
/// resource type.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GrantTokenResourcesPayload {
    /// Specific channels permissions for `channel`-based endpoints access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    channels: Option<HashMap<String, u8>>,

    /// Specific channel groups permissions for `channel group`-based endpoints
    /// access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    groups: Option<HashMap<String, u8>>,

    /// Specific `userId` permissions for `userId`-based endpoints access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    uuids: Option<HashMap<String, u8>>,
}

/// Token permissions.
///
/// This type used by [`GrantTokenPayload`] to store information about requested
/// token permissions.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GrantTokenPermissionsPayload<'request> {
    /// A user ID, which is authorized to use the token to make API requests to
    /// PubNub.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub uuid: &'request Option<String>,

    /// Extra metadata to be published with the request. Values must be scalar only.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub meta: &'request Option<HashMap<String, MetaValue>>,

    /// List of permissions mapped to resource identifiers.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub resources: Option<GrantTokenResourcesPayload>,

    /// List of permissions mapped to RegExp match expressions.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub patterns: Option<GrantTokenResourcesPayload>,
}

/// Payload for grant token operation.
///
/// A list of resource names and patterns and permissions which should be
/// granted for them is contained in the payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GrantTokenPayload<'request> {
    /// How long (in minutes) the generated token should be valid.
    pub ttl: usize,

    /// Permissions which should be granted to token.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub permissions: Option<GrantTokenPermissionsPayload<'request>>,
}

impl<'request> GrantTokenPayload<'request> {
    /// Create request payload.
    ///
    /// The information provided to the request builder is used to compose the
    /// final payload, which should then be sent to the PubNub API endpoint.
    pub(super) fn new<T, S, D>(request: &'request GrantTokenRequest<'_, T, S, D>) -> Self
    where
        S: for<'se, 'rq> Serializer<'se, GrantTokenPayload<'rq>>,
        D: for<'ds> Deserializer<'ds, GrantTokenResponseBody>,
    {
        GrantTokenPayload {
            ttl: request.ttl,
            permissions: Some(GrantTokenPermissionsPayload {
                patterns: resource_permissions(&request.patterns),
                resources: resource_permissions(&request.resources),
                uuid: &request.authorized_user_id,
                meta: &request.meta,
            }),
        }
    }
}

/// Extract permissions for list of resources.
fn resource_permissions(
    resources: &Option<&[Box<dyn Permission>]>,
) -> Option<GrantTokenResourcesPayload> {
    resources.map(|res| {
        let mut channels: HashMap<String, u8> = HashMap::new();
        let mut groups: HashMap<String, u8> = HashMap::new();
        let mut uuids: HashMap<String, u8> = HashMap::new();
        res.iter().for_each(|perm| match perm.resource_type() {
            ResourceType::Channel => {
                channels.insert(perm.id().to_string(), *perm.value());
            }
            ResourceType::ChannelGroup => {
                groups.insert(perm.id().to_string(), *perm.value());
            }
            ResourceType::UserId => {
                uuids.insert(perm.id().to_string(), *perm.value());
            }
        });

        GrantTokenResourcesPayload {
            channels: (!channels.is_empty()).then_some(channels),
            groups: (!groups.is_empty()).then_some(groups),
            uuids: (!uuids.is_empty()).then_some(uuids),
        }
    })
}
