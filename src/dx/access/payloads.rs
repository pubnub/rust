//! Request payloads module.
//!
//! This module contains: [`Permission`] struct and it's implementation of
//! [`ChannelPermission`],  [`ChannelGroupPermission`] and [`UserIdPermission`]
//! traits.

use crate::{
    core::Serializer,
    dx::access::{permissions::*, types::MetaValue, GrantTokenRequest},
    lib::{
        alloc::{
            boxed::Box,
            string::{String, ToString},
        },
        collections::HashMap,
    },
};

/// Resource and pattern-based permissions' payload.
///
/// This type used by [`GrantTokenPayload`] to store permissions for specific
/// resource type.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantTokenResourcesPayload {
    /// Specific channels permissions for `channel`-based endpoints access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub channels: Option<HashMap<String, u8>>,

    /// Specific channel groups permissions for `channel group`-based endpoints
    /// access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub groups: Option<HashMap<String, u8>>,

    /// Specific `userId` permissions for `userId`-based endpoints access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub uuids: Option<HashMap<String, u8>>,
}

/// Categories permissions payload.
///
/// This type used by [`GrantTokenPayload`] to store permissions for specific
/// resource type.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrantTokenCategoriesPayload {
    /// Category permissions for `channels` enumeration endpoint access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub channels: Option<u8>,

    /// Category permissions for `uuids` enumeration endpoint access.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub uuids: Option<u8>,
}

/// Token permissions.
///
/// This type used by [`GrantTokenPayload`] to store information about requested
/// token permissions.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTokenPermissionsPayload<'request> {
    /// A user ID, which is authorized to use the token to make API requests to
    /// PubNub.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub uuid: &'request Option<String>,

    /// Extra metadata to be published with the request. Values must be scalar
    /// only.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub meta: &'request Option<HashMap<String, MetaValue>>,

    /// List of permissions mapped to resource identifiers.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub resources: Option<GrantTokenResourcesPayload>,

    /// List of permissions mapped to RegExp match expressions.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub patterns: Option<GrantTokenResourcesPayload>,

    /// List of permissions for enumerate-able resource categories.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub categories: Option<GrantTokenCategoriesPayload>,
}

/// Payload for grant token operation.
///
/// A list of resource names and patterns and permissions which should be
/// granted for them is contained in the payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
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
    {
        GrantTokenPayload {
            ttl: request.ttl,
            permissions: Some(GrantTokenPermissionsPayload {
                patterns: resource_permissions(&request.patterns),
                resources: resource_permissions(&request.resources),
                categories: categories_permissions(&request.categories),
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
            resource_type @ (ResourceType::ChannelsCategory | ResourceType::UuidsCategory) => {
                log::error!(
                    "Categories permissions can't be used for `resources` or `patterns`, \
                     skipping `{:?}`",
                    resource_type
                );
            }
        });

        GrantTokenResourcesPayload {
            channels: (!channels.is_empty()).then_some(channels),
            groups: (!groups.is_empty()).then_some(groups),
            uuids: (!uuids.is_empty()).then_some(uuids),
        }
    })
}

/// Extract permissions for categories.
fn categories_permissions(
    resources: &Option<&[Box<dyn Permission>]>,
) -> Option<GrantTokenCategoriesPayload> {
    resources.map(|res| {
        let mut channels_category_perm: Option<u8> = None;
        let mut uuids_category_perm: Option<u8> = None;
        res.iter().for_each(|perm| match perm.resource_type() {
            ResourceType::ChannelsCategory => fold_category_permission(
                &mut channels_category_perm,
                perm.resource_type(),
                perm.value(),
            ),
            ResourceType::UuidsCategory => fold_category_permission(
                &mut uuids_category_perm,
                perm.resource_type(),
                perm.value(),
            ),
            resource_type => log::error!(
                "Only `ResourceType::ChannelsCategory` or `ResourceType::UuidsCategory` accepted \
                 for categories, but received `{:?}`",
                resource_type
            ),
        });

        GrantTokenCategoriesPayload {
            channels: channels_category_perm.filter(|perm| *perm > 0),
            uuids: uuids_category_perm.filter(|perm| *perm > 0),
        }
    })
}

/// Combine category permission with previously provided ones.
///
/// Permissions are bit flags, so repeated entries are merged using bitwise OR.
fn fold_category_permission(category: &mut Option<u8>, resource_type: ResourceType, value: &u8) {
    if category.is_some() {
        log::warn!(
            "Multiple `{:?}` permissions provided, combining them into one",
            resource_type
        );
    }
    *category = Some(category.unwrap_or(0) | *value);
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::dx::access::permissions::{self, GET};

    #[test]
    fn build_categories_payload_with_both_categories() {
        let perms: [Box<dyn Permission>; 2] = [
            permissions::channels_category().get(),
            permissions::uuids_category().get(),
        ];
        let payload = categories_permissions(&Some(&perms)).unwrap();
        assert_eq!(payload.channels, Some(GET));
        assert_eq!(payload.uuids, Some(GET));
    }

    #[test]
    fn omit_category_without_permission() {
        let perms: [Box<dyn Permission>; 1] = [permissions::channels_category().get()];
        let payload = categories_permissions(&Some(&perms)).unwrap();
        assert_eq!(payload.channels, Some(GET));
        assert_eq!(payload.uuids, None);
    }

    #[test]
    fn not_treat_category_as_channel_resource() {
        let perms: [Box<dyn Permission>; 1] = [permissions::channels_category().get()];
        let payload = resource_permissions(&Some(&perms)).unwrap();
        assert!(payload.channels.is_none());
    }

    #[test]
    fn omit_categories_from_resources() {
        let perms: [Box<dyn Permission>; 2] = [
            permissions::channels_category().get(),
            permissions::uuids_category().get(),
        ];
        let payload = resource_permissions(&Some(&perms)).unwrap();
        assert!(payload.channels.is_none());
        assert!(payload.groups.is_none());
        assert!(payload.uuids.is_none());
    }

    #[test]
    fn combine_repeated_channels_category_permissions_with_bitwise_or() {
        let perms: [Box<dyn Permission>; 2] = [
            permissions::channels_category().get(),
            permissions::channels_category().get(),
        ];
        let payload = categories_permissions(&Some(&perms)).unwrap();
        assert_eq!(payload.channels, Some(GET));
        assert_eq!(payload.uuids, None);
    }

    #[test]
    fn ignore_resource_type_which_is_not_category() {
        let perms: [Box<dyn Permission>; 2] = [
            permissions::channels_category().get(),
            permissions::channel("my-channel").get(),
        ];
        let payload = categories_permissions(&Some(&perms)).unwrap();
        assert_eq!(payload.channels, Some(GET));
        assert_eq!(payload.uuids, None);
    }
}
