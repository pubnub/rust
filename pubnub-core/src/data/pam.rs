//! PAMv3 related types.

use super::object::Object;
use std::collections::HashMap;
use std::fmt::Debug;

/// Permissions bitmask. Values can be combined with a bitwise OR operation.
///
/// |Name    |Value (binary)|Value (hex)|Value (dec)|Description                                      |
/// |--------|--------------|-----------|-----------|-------------------------------------------------|
/// |`READ`  |`0b0000_0001` |`0x01`     |`1`        |Applies to Subscribe, History, Presence, Objects |
/// |`WRITE` |`0b0000_0010` |`0x02`     |`2`        |Applies to Publish, Objects                      |
/// |`MANAGE`|`0b0000_0100` |`0x04`     |`4`        |Applies to Channel-Groups, Objects               |
/// |`DELETE`|`0b0000_1000` |`0x08`     |`8`        |Applies to History                               |
/// |`CREATE`|`0b0001_0000` |`0x10`     |`16`       |Applies to Objects                               |
///
/// ## Permissions matrix:
///
/// |Resource type|Permission|API                    |Allowances                                         |
/// |-------------|----------|-----------------------|---------------------------------------------------|
/// |`channels`   |`READ`    |Subscribe              |Receiving messages on a channel                    |
/// |`channels`   |`READ`    |Presence Here Now      |Listing UUIDs subscribed to a channel              |
/// |`channels`   |`READ`    |Presence User State    |Set/get state on a channel                         |
/// |`channels`   |`READ`    |Push; Add Device       |Adding a device to a channel for push notifications|
/// |`channels`   |`READ`    |History                |Receiving historical messages on a channel         |
/// |`channels`   |`DELETE`  |History; Delete        |Deleting historical messages on a channel          |
/// |`channels`   |`WRITE`   |Publish                |Sending messages on a channel                      |
/// |`channels`   |`WRITE`   |Signal                 |Sending signals on a channel                       |
/// |`groups`     |`READ`    |Subscribe              |Receiving messages on a channel-group              |
/// |`groups`     |`READ`    |Presence Here Now      |Listing UUIDs subscribed to a channel-group        |
/// |`groups`     |`READ`    |Presence User State    |Set/get state on a channel-group                   |
/// |`groups`     |`READ`    |Groups; List           |Listing all channels in a channel-group            |
/// |`groups`     |`MANAGE`  |Groups; Add Channels   |Adding channels to a channel-group                 |
/// |`groups`     |`MANAGE`  |Groups; Remove Channels|Removing channels from a channel-group             |
/// |`groups`     |`MANAGE`  |Delete Group           |Deleting a channel-group                           |
/// |`users`      |`CREATE`  |User; Create           |Creating a user by `UserID`                        |
/// |`users`      |`DELETE`  |User; Delete           |Deleting a user and all of its space memberships   |
/// |`users`      |`MANAGE`  |User; Add membership   |Adding space membership for a user                 |
/// |`users`      |`READ`    |User; Read             |Reading a user's information and space memberships |
/// |`users`      |`WRITE`   |User; Update           |Updating a user's information                      |
/// |`spaces`     |`CREATE`  |Space; Create          |Creating a space by `SpaceID`                      |
/// |`spaces`     |`DELETE`  |Space; Delete          |Deleting a space and all of its members            |
/// |`spaces`     |`MANAGE`  |Space; Add members     |Adding members to a space                          |
/// |`spaces`     |`READ`    |Space; Read            |Reading a space's information and member users     |
/// |`spaces`     |`WRITE`   |Space; Update          |Updating a space's information                     |
/// |`spaces`     |`MANAGE`  |Space; User Memberships|Adding and removing members from a space           |
///
/// **⚠️ Use of undocumented bitmask values or combinations with resource
/// types is considered undefined behavior; Using undefined behavior in
/// grant requests or within tokens passed to any PubNub REST API are
/// allowed to break in unexpected ways, including spawning
/// ["nasal demons"](http://www.catb.org/jargon/html/N/nasal-demons.html).**
// TODO: use bitflags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitMask(pub u64);

/// The PAMv3 grant request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantBody {
    /// The total duration (in minutes) that the token will remain valid
    /// The minimum ttl allowed is 1 minute. The maximum ttl allowed is 43,200
    /// minutes (equivalent to 30 days).
    // TODO: use a constrained type here.
    pub ttl: u32,

    /// Permissions object schema.
    pub permissions: Permissions,
}

/// Grant permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    /// A mapping of resource types to resource IDs.
    pub resources: Resources,

    /// A mapping of resource types to regular expressions.
    pub patterns: Patterns,

    /// The meta mapping is available for arbitrary key-value pairs, to use
    /// as your application sees fit. Beware that the `meta` object is copied
    /// into the token verbatim; potentially being a significant source of
    /// "token bloat".
    ///
    /// This mapping may be used for identity/authentication purposes,
    /// restricting token use (in the "public key use" sense as defined by JWK),
    /// or exclusions/exceptions.
    ///
    /// PubNub reserves all keys beginning with the three-character prefix `pn-`
    /// for future purposes.
    ///
    /// Use of undocumented reserved meta fields is considered undefined
    /// behavior
    pub meta: Object,
}

/// A mapping of resource types to permissions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resources {
    /// A shallow mapping of channel names to permissions.
    pub channels: HashMap<String, BitMask>,

    /// A shallow mapping of channel groups to permissions.
    pub groups: HashMap<String, BitMask>,

    /// A shallow mapping of user IDs to permissions.
    pub users: HashMap<String, BitMask>,

    /// A shallow mapping of space IDs to permissions.
    pub spaces: HashMap<String, BitMask>,
}

type PatternRegex = String;

/// A mapping of resource types as regular expressions to permissions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Patterns {
    /// A shallow mapping of channel regular expressions to permissions.
    pub channels: HashMap<PatternRegex, BitMask>,

    /// A shallow mapping of channel-group regular expressions to permissions.
    pub groups: HashMap<PatternRegex, BitMask>,

    /// A shallow mapping of user ID regular expressions to permissions.
    pub users: HashMap<PatternRegex, BitMask>,

    /// A shallow mapping of space ID regular expressions to permissions.
    pub spaces: HashMap<PatternRegex, BitMask>,
}
