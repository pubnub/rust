//! Presence API related types.
use super::channel;
use super::object::Object;
use super::uuid::UUID;
use std::collections::HashMap;

pub mod respond_with {
    //! Type system level flags to specialize the response types.

    /// Return occupancy only.
    #[derive(Debug, Clone, Copy)]
    pub struct OccupancyOnly;
    /// Return occupancy and UUIDs of the users.
    #[derive(Debug, Clone, Copy)]
    pub struct OccupancyAndUUIDs;
    /// Return cooupance, UUIDs of the users and the related states.
    #[derive(Debug, Clone, Copy)]
    pub struct Full;

    /// A trait that bounds type system level flag to an actual type
    /// representing the response structure.
    pub trait RespondWith {
        /// The response type that will be produced for the particular
        /// type system level flag type.
        type Response;
    }

    use super::{ChannelInfo, ChannelInfoWithOccupants, ChannelOccupantFullDetails, UUID};

    impl RespondWith for OccupancyOnly {
        type Response = ChannelInfo;
    }
    impl RespondWith for OccupancyAndUUIDs {
        type Response = ChannelInfoWithOccupants<UUID>;
    }
    impl RespondWith for Full {
        type Response = ChannelInfoWithOccupants<ChannelOccupantFullDetails>;
    }
}

/// Represents the channel info returned from here now and global here now
/// family of calls where only the occupants amount is returned.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelInfo {
    /// The amount of devices on a channel.
    pub occupancy: u64,
}

/// Represents the channel info returned from here now and global here now
/// family of calls where the occupants amount and per-cooupant details
/// are returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelInfoWithOccupants<T> {
    /// The amount of devices on a channel.
    pub occupancy: u64,

    /// Info on the occupants.
    pub occupants: Vec<T>,
}

/// Represents the channel occupant details when the full data is requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelOccupantFullDetails {
    /// The UUID of the occupant.
    pub uuid: UUID,

    /// The state data of the occupant.
    pub state: Object,
}

/// Represents the global info returned from global here now call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalInfo<T: respond_with::RespondWith> {
    /// The total number of channels.
    pub total_channels: u64,

    /// The total occupancy across all the channels.
    pub total_occupancy: u64,

    /// Per-channel info.
    pub channels: HashMap<channel::Name, T::Response>,
}
