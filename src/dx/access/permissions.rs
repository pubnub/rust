//! Resource permissions module.
//!
//! This module contains: [`Permission`] struct and it's implementation of
//! [`ChannelPermission`],  [`ChannelGroupPermission`] and [`UserIdPermission`]
//! traits.

/// Resource **read** permissions.
const READ: u8 = 0b0000_0001;
/// Resource **write** permissions.
const WRITE: u8 = 0b0000_0010;
/// Resource **manage** permissions.
const MANAGE: u8 = 0b0000_0100;
/// Resource **delete** permissions.
const DELETE: u8 = 0b0000_1000;
/// Resource **get** permissions.
const GET: u8 = 0b0010_0000;
/// Resource **update** permissions.
const UPDATE: u8 = 0b0100_0000;
/// Resource **join** permissions.
const JOIN: u8 = 0b1000_0000;

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
}

/// `Channel`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `channels`.
pub trait ChannelPermission {
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
    fn read(self) -> Self;

    /// Resource **write** permissions.
    ///
    /// This permission for channel allows to:
    /// * publish message
    /// * add message action
    /// * send signal
    /// * send file
    fn write(self) -> Self;

    /// Resource **delete** permissions.
    ///
    /// This permission for channel allows to:
    /// * delete messages
    /// * delete message action
    /// * delete file
    /// * delete channel metadata
    fn delete(self) -> Self;

    /// Resource **get** permissions.
    ///
    /// This permission for channel allows to:
    /// * get channel metadata
    /// * get channel members
    fn get(self) -> Self;

    /// Resource **update** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel metadata
    fn update(self) -> Self;

    /// Resource **manage** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel members
    /// * remove channel members
    fn manage(self) -> Self;

    /// Resource **join** permissions.
    ///
    /// This permission for channel allows to:
    /// * set channel memberships
    /// * remove channel memberships
    fn join(self) -> Self;
}

/// `Channel group`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `channel groups`.
pub trait ChannelGroupPermission {
    /// Resource **read** permissions.
    ///
    /// This permission for channel groups (including `channel-group-pnpres`)
    /// allows to:
    /// * subscribe
    fn read(self) -> Self;

    /// Resource **manage** permissions.
    ///
    /// This permission for channel groups allows to:
    /// * add channels to channel group
    /// * remove channels from channel group
    /// * list channels in channel group
    /// * remove channel group
    fn manage(self) -> Self;
}

/// `UserId`-based endpoint permissions.
///
/// Trait contains methods to configure permissions to access endpoints related
/// to `UserId`.
pub trait UserIdPermission {
    /// Resource **get** permissions.
    ///
    /// This permission for user ids allows to:
    /// * get user metadata
    /// * get channel memberships
    fn get(self) -> Self;

    /// Resource **update** permissions.
    ///
    /// This permission for user ids allows to:
    /// * set user metadata
    /// * set channel memberships
    /// * remove channel memberships
    fn update(self) -> Self;

    /// Resource **delete** permissions.
    ///
    /// This permission for user ids allows to:
    /// * delete user metadata
    fn delete(self) -> Self;
}

/// `Channel` permission.
///
/// Single `channel` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, Channel, Permission, ChannelPermission};
/// #
/// let channel_permission = // Channel
/// #    permissions::channel("my-channel")
///     .get()
///     .manage();
/// ```
pub struct Channel {
    name: String,
    bits: u8,
}

/// `Channel group` permission.
///
/// Single `channel group` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelGroup, ChannelGroupPermission};
/// #
/// let channel_permission = // ChannelGroup
/// #    permissions::channel_group("my-channel-group")
///     .read();
/// ```
pub struct ChannelGroup {
    name: String,
    bits: u8,
}

/// `UserId` permission.
///
/// Single `userId` permission information.
///
/// # Example
/// ```rust, no_run
/// # use pubnub::dx::access::permissions::{self, Permission, UserId, UserIdPermission};
/// #
/// let channel_permission = // UserId
/// #    permissions::user_id("my-user-id")
///     .update();
/// ```
pub struct UserId {
    id: String,
    bits: u8,
}

impl Permission for Channel {
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

impl Permission for ChannelGroup {
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

impl Permission for UserId {
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

impl ChannelPermission for Box<Channel> {
    fn read(mut self) -> Self {
        self.bits |= READ;
        self
    }

    fn write(mut self) -> Self {
        self.bits |= WRITE;
        self
    }

    fn delete(mut self) -> Self {
        self.bits |= DELETE;
        self
    }

    fn get(mut self) -> Self {
        self.bits |= GET;
        self
    }

    fn update(mut self) -> Self {
        self.bits |= UPDATE;
        self
    }

    fn manage(mut self) -> Self {
        self.bits |= MANAGE;
        self
    }

    fn join(mut self) -> Self {
        self.bits |= JOIN;
        self
    }
}

impl ChannelGroupPermission for Box<ChannelGroup> {
    fn read(mut self) -> Self {
        self.bits |= READ;
        self
    }
    fn manage(mut self) -> Self {
        self.bits |= MANAGE;
        self
    }
}

impl UserIdPermission for Box<UserId> {
    fn get(mut self) -> Self {
        self.bits |= GET;
        self
    }

    fn update(mut self) -> Self {
        self.bits |= UPDATE;
        self
    }

    fn delete(mut self) -> Self {
        self.bits |= DELETE;
        self
    }
}

/// Grant permission to the `channel`.
///
/// Create a `channel` permission information object that can be used to specify
/// required permissions.
///
/// # Example
/// ```rust
/// # use pubnub::dx::access::permissions::{self, Channel, Permission, ChannelPermission};
/// #
/// let channel_permission = permissions::channel("my-channel").get().manage();
/// # assert_eq!(channel_permission.value(), &0b0010_0100);
/// ```
pub fn channel<N>(name: N) -> Box<Channel>
where
    N: Into<String>,
{
    Box::new(Channel {
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
/// # use pubnub::dx::access::permissions::{self, Permission, ChannelGroup, ChannelGroupPermission};
/// #
/// let channel_group_permission = permissions::channel_group("my-channel-group")
///     .read();
/// # assert_eq!(channel_group_permission.value(), &0b0000_0001);
/// ```
pub fn channel_group<N>(name: N) -> Box<ChannelGroup>
where
    N: Into<String>,
{
    Box::new(ChannelGroup {
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
/// # use pubnub::dx::access::permissions::{self, Permission, UserId, UserIdPermission};
/// #
/// let channel_group_permission = permissions::user_id("my-user-id")
///     .update();
/// # assert_eq!(channel_group_permission.value(), &0b0100_0000);
/// ```
pub fn user_id<I>(id: I) -> Box<UserId>
where
    I: Into<String>,
{
    Box::new(UserId {
        id: id.into(),
        bits: 0,
    })
}

#[cfg(test)]
mod it_should {
    use super::*;

    #[test]
    fn set_proper_read_permission_bits() {
        assert_eq!(channel("test").read().value(), &0b0000_0001);
    }

    #[test]
    fn set_proper_write_permission_bits() {
        assert_eq!(channel("test").write().value(), &0b0000_0010);
    }

    #[test]
    fn set_proper_manage_permission_bits() {
        assert_eq!(channel("test").manage().value(), &0b0000_0100);
    }

    #[test]
    fn set_proper_delete_permission_bits() {
        assert_eq!(channel("test").delete().value(), &0b0000_1000);
    }

    #[test]
    fn set_proper_get_permission_bits() {
        assert_eq!(channel("test").get().value(), &0b0010_0000);
    }

    #[test]
    fn set_proper_update_permission_bits() {
        assert_eq!(channel("test").update().value(), &0b0100_0000);
    }

    #[test]
    fn set_proper_join_permission_bits() {
        assert_eq!(channel("test").join().value(), &0b1000_0000);
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
}
