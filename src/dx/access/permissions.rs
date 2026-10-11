//! Resource permissions module.
//!
//! This module contains: [`Permission`] struct and it's implementation of
//! [`ChannelPermission`],  [`ChannelGroupPermission`],  [`UserIdPermission`],
//! [`ChannelsCategoryPermission`], and [`UuidsCategoryPermission`]
//! traits.

use crate::lib::{alloc::boxed::Box, alloc::string::String};

/// Resource **read** permissions.
pub(super) const READ: u8 = 0b0000_0001;
/// Resource **write** permissions.
pub(super) const WRITE: u8 = 0b0000_0010;
/// Resource **manage** permissions.
pub(super) const MANAGE: u8 = 0b0000_0100;
/// Resource **delete** permissions.
pub(super) const DELETE: u8 = 0b0000_1000;
/// Resource **get** permissions.
pub(super) const GET: u8 = 0b0010_0000;
/// Resource **update** permissions.
pub(super) const UPDATE: u8 = 0b0100_0000;
/// Resource **join** permissions.
pub(super) const JOIN: u8 = 0b1000_0000;

/// Resource-based endpoint access permission.
///
/// When [`PubNub Access Manager`] is enabled, access to resources becomes
/// restricted and only clients with the corresponding access token may access
/// allowed resources within the provided permissions.
///
/// [`PubNub Access Manager`]: https://www.pubnub.com/docs/general/security/access-control
pub trait Permission {
    /// The name or pattern of the channel name for which permissions were
    /// specified.
    fn id(&self) -> &String;

    /// Calculated channel permissions value.
    fn value(&self) -> &u8;

    /// Actual type of resource for which permission has been specified.
    fn resource_type(&self) -> ResourceType;
}

/// Type of resource for permissions.
pub enum ResourceType {
    /// `Channel`-based resource permissions.
    Channel,
    /// `Channel group`-based resource permissions.
    ChannelGroup,
    /// `UserId`-based resource permissions.
    UserId,
    /// Channels category permissions.
    ChannelsCategory,
    /// Uuids category permissions.
    UuidsCategory,
}

/// `Channel` permission.
///
/// Single `channel` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, ChannelPermission, Permission};
/// #
/// let channel_permission = // Channel
/// #    permissions::channel("my-channel")
///     .get()
///     .manage();
/// ```
#[derive(Debug)]
pub struct ChannelPermission {
    /// Name of channel for which permission level specified.
    pub name: String,

    /// Bitmask with configured permission level.
    pub bits: u8,
}

/// `Channel group` permission.
///
/// Single `channel group` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelGroupPermission};
/// #
/// let channel_permission = // ChannelGroup
/// #    permissions::channel_group("my-channel-group")
///     .read();
/// ```
#[derive(Debug)]
pub struct ChannelGroupPermission {
    /// Name of channel group for which permission level specified.
    pub name: String,

    /// Bitmask with configured permission level.
    pub bits: u8,
}

/// `UserId` permission.
///
/// Single `userId` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, Permission, UserIdPermission};
/// #
/// let channel_permission = // UserId
/// #    permissions::user_id("my-user-id")
///     .update();
/// ```
#[derive(Debug)]
pub struct UserIdPermission {
    /// User id for which permission level specified.
    pub id: String,

    /// Bitmask with configured permission level.
    pub bits: u8,
}

/// `Channels category` permission.
///
/// Single `channels category` category permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, ChannelsCategoryPermission, Permission};
/// #
/// let channels_category_permission = // Channels category
/// #    permissions::channels_category()
///     .get();
/// ```
#[derive(Debug)]
pub struct ChannelsCategoryPermission {
    /// Name of channels category name for which permission level specified.
    name: String,

    /// Bitmask with configured permission level.
    pub bits: u8,
}

/// `Uuids category` permission.
///
/// Single `uuids category` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, UuidsCategoryPermission, Permission};
/// #
/// let uuids_category_permission = // Uuids category
/// #    permissions::uuids_category()
///     .get();
/// ```
#[derive(Debug)]
pub struct UuidsCategoryPermission {
    /// Name of uuids category name for which permission level specified.
    name: String,

    /// Bitmask with configured permission level.
    pub bits: u8,
}

/// `Channel`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `channels`.
impl ChannelPermission {
    /// Resource **read** permissions.
    ///
    /// This permission for channel (including `channel-pnpres`) allows to:
    /// * subscribe
    /// * retrieve list of subscribers
    /// * get state associated with `userId`
    /// * associate state with `userId`
    /// * fetch messages history
    /// * fetch messages history with message actions
    /// * fetch message actions
    /// * retrieve messages count
    /// * list files
    /// * download files
    /// * register for push notifications
    /// * unregister from push notifications
    pub fn read(mut self) -> Box<Self> {
        self.bits |= READ;
        Box::new(self)
    }

    /// Resource **write** permissions.
    ///
    /// This permission for channel allows to:
    /// * publish message
    /// * add message action
    /// * send signal
    /// * send file
    pub fn write(mut self) -> Box<Self> {
        self.bits |= WRITE;
        Box::new(self)
    }

    /// Resource **delete** permissions.
    ///
    /// This permission for channel allows to:
    /// * delete messages
    /// * delete message action
    /// * delete file
    /// * delete channel metadata
    pub fn delete(mut self) -> Box<Self> {
        self.bits |= DELETE;
        Box::new(self)
    }

    /// Resource **get** permissions.
    ///
    /// This permission for channel allows to:
    /// * get channel metadata
    /// * get channel members
    pub fn get(mut self) -> Box<Self> {
        self.bits |= GET;
        Box::new(self)
    }

    /// Resource **update** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel metadata
    pub fn update(mut self) -> Box<Self> {
        self.bits |= UPDATE;
        Box::new(self)
    }

    /// Resource **manage** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel members
    /// * remove channel members
    pub fn manage(mut self) -> Box<Self> {
        self.bits |= MANAGE;
        Box::new(self)
    }

    /// Resource **join** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel memberships
    /// * remove channel memberships
    pub fn join(mut self) -> Box<Self> {
        self.bits |= JOIN;
        Box::new(self)
    }
}

impl Permission for ChannelPermission {
    fn id(&self) -> &String {
        &self.name
    }

    fn value(&self) -> &u8 {
        &self.bits
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::Channel
    }
}

/// `Channel group`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `channel groups`.
impl ChannelGroupPermission {
    /// Resource **read** permissions.
    ///
    /// This permission for channel groups (including `channel-group-pnpres`)
    /// allows to:
    /// * subscribe
    pub fn read(mut self) -> Box<Self> {
        self.bits |= READ;
        Box::new(self)
    }

    /// Resource **manage** permissions.
    ///
    /// This permission for channel groups allows to:
    /// * add channels to channel group
    /// * remove channels from channel group
    /// * list channels in channel group
    /// * remove channel group
    pub fn manage(mut self) -> Box<Self> {
        self.bits |= MANAGE;
        Box::new(self)
    }
}

impl Permission for ChannelGroupPermission {
    fn id(&self) -> &String {
        &self.name
    }

    fn value(&self) -> &u8 {
        &self.bits
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::ChannelGroup
    }
}

/// `UserId`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `UserId`.
impl UserIdPermission {
    /// Resource **get** permissions.
    ///
    /// This permission for user ids allows to:
    /// * get user metadata
    /// * get channel memberships
    pub fn get(mut self) -> Box<Self> {
        self.bits |= GET;
        Box::new(self)
    }

    /// Resource **update** permissions.
    ///
    /// This permission for user ids allows to:
    /// * set user metadata
    /// * set channel memberships
    /// * remove channel memberships
    pub fn update(mut self) -> Box<Self> {
        self.bits |= UPDATE;
        Box::new(self)
    }

    /// Resource **delete** permissions.
    ///
    /// This permission for user ids allows to:
    /// * delete user metadata
    pub fn delete(mut self) -> Box<Self> {
        self.bits |= DELETE;
        Box::new(self)
    }
}

impl Permission for UserIdPermission {
    fn id(&self) -> &String {
        &self.id
    }

    fn value(&self) -> &u8 {
        &self.bits
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::UserId
    }
}

/// `Channels category` enumeration endpoint permissions.
impl ChannelsCategoryPermission {
    /// Resource **get** permissions.
    ///
    /// This permission for channels category allows to:
    /// * get all channels metadata
    pub fn get(mut self) -> Box<Self> {
        self.bits |= GET;
        Box::new(self)
    }
}

impl Permission for ChannelsCategoryPermission {
    fn id(&self) -> &String {
        &self.name
    }

    fn value(&self) -> &u8 {
        &self.bits
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::ChannelsCategory
    }
}

/// `Uuids category` enumeration endpoint permissions.
impl UuidsCategoryPermission {
    /// Resource **get** permissions.
    ///
    /// This permission for uuids category allows to:
    /// * get all uuids metadata
    pub fn get(mut self) -> Box<Self> {
        self.bits |= GET;
        Box::new(self)
    }
}

impl Permission for UuidsCategoryPermission {
    fn id(&self) -> &String {
        &self.name
    }

    fn value(&self) -> &u8 {
        &self.bits
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::UuidsCategory
    }
}

/// Grant permission to the `channel`.
///
/// Create a `channel` permission information object that can be used to specify
/// required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelPermission};
/// #
/// let channel_permission = permissions::channel("my-channel").get().manage();
/// # assert_eq!(channel_permission.value(), &0b0010_0100);
/// ```
pub fn channel<N>(name: N) -> Box<ChannelPermission>
where
    N: Into<String>,
{
    Box::new(ChannelPermission {
        name: name.into(),
        bits: 0,
    })
}

/// Grant permission to the `channel group`.
///
/// Create a `channel group` permission information object that can be used to
/// specify required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelGroupPermission};
/// #
/// let channel_group_permission = permissions::channel_group("my-channel-group")
///     .read();
/// # assert_eq!(channel_group_permission.value(), &0b0000_0001);
/// ```
pub fn channel_group<N>(name: N) -> Box<ChannelGroupPermission>
where
    N: Into<String>,
{
    Box::new(ChannelGroupPermission {
        name: name.into(),
        bits: 0,
    })
}

/// Grant permission to the `userId`.
///
/// Create a `userId` permission information object that can be used to specify
/// required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Permission, UserIdPermission};
/// #
/// let channel_group_permission = permissions::user_id("my-user-id")
///     .update();
/// # assert_eq!(channel_group_permission.value(), &0b0100_0000);
/// ```
pub fn user_id<I>(id: I) -> Box<UserIdPermission>
where
    I: Into<String>,
{
    Box::new(UserIdPermission {
        id: id.into(),
        bits: 0,
    })
}

/// Grant permission to the `channel category`.
///
/// Create a `channel category` permission information object that can be used
/// to specify required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelsCategoryPermission};
/// #
/// let channels_category_permission = permissions::channels_category().get();
/// # assert_eq!(channels_category_permission.value(), &0b0010_0000);
/// ```
pub fn channels_category() -> Box<ChannelsCategoryPermission> {
    Box::new(ChannelsCategoryPermission {
        name: String::from("channels"),
        bits: 0,
    })
}

/// Grant permission to the `uuid category`.
///
/// Create a `uuid category` permission information object that can be used
/// to specify required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Permission, UuidsCategoryPermission};
/// #
/// let uuids_category_permission = permissions::uuids_category().get();
/// # assert_eq!(uuids_category_permission.value(), &0b0010_0000);
/// ```
pub fn uuids_category() -> Box<UuidsCategoryPermission> {
    Box::new(UuidsCategoryPermission {
        name: String::from("uuids"),
        bits: 0,
    })
}

#[cfg(test)]
mod it_should {
    use super::*;
    use test_case::test_case;

    #[test_case(channel("test").read().value(), &0b0000_0001 ; "compare read permission")]
    #[test_case(channel("test").write().value(), &0b0000_0010 ; "compare write permission")]
    #[test_case(channel("test").manage().value(), &0b0000_0100 ; "compare manage permission")]
    #[test_case(channel("test").delete().value(), &0b0000_1000 ; "compare delete permission")]
    #[test_case(channel("test").get().value(), &0b0010_0000 ; "compare get permission")]
    #[test_case(channel("test").update().value(), &0b0100_0000 ; "compare update permission")]
    #[test_case(channel("test").join().value(), &0b1000_0000 ; "compare join permission")]
    fn bits_properly_set(expected: &u8, actual: &u8) {
        assert_eq!(expected, actual);
    }

    #[test]
    fn create_channel_with_read_delete_join_permission() {
        let channel_name = "test-channel";
        let permission = channel(channel_name).read().delete().join();
        assert_eq!(permission.value(), &0b1000_1001);
        assert_eq!(permission.id(), channel_name);
    }

    #[test]
    fn create_channel_group_manage_permission() {
        let channel_group_name = "test-channel-group";
        let permission = channel_group(channel_group_name).manage();
        assert_eq!(permission.value(), &0b0000_0100);
        assert_eq!(permission.id(), channel_group_name);
    }

    #[test]
    fn create_user_id_get_update_permission() {
        let user_id_value = "test-user-id";
        let permission = user_id(user_id_value).get().update();
        assert_eq!(permission.value(), &0b0110_0000);
        assert_eq!(permission.id(), user_id_value);
    }

    #[test]
    fn create_channels_category_get_permission() {
        let permission = channels_category().get();
        assert_eq!(permission.value(), &0b0010_0000);
        assert_eq!(permission.id(), "channels");
    }

    #[test]
    fn create_uuids_category_get_permission() {
        let permission = uuids_category().get();
        assert_eq!(permission.value(), &0b0010_0000);
        assert_eq!(permission.id(), "uuids");
    }
}
