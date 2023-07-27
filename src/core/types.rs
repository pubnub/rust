//! # Common PubNub module tyoes
//!
//! The module contains [`ScalarValue`] type used in various operations and data types.

use crate::lib::alloc::string::String;

/// Scalar values for flattened [`HashMap`].
///
/// Some endpoints only require [`HashMap`], which should not have nested
/// collections. This requirement is implemented through this type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(untagged))]
pub enum ScalarValue {
    /// `String` value stored for specific key in [`HashMap`].
    String(String),

    /// `Boolean` value stored for specific key in [`HashMap`].
    Boolean(bool),

    /// `signed 8-bit` value stored for specific key in [`HashMap`].
    Signed8(i8),

    /// `unsigned 8-bit` value stored for specific key in [`HashMap`].
    Unsigned8(u8),

    /// `signed 16-bit` value stored for specific key in [`HashMap`].
    Signed16(i16),

    /// `unsigned 16-bit` value stored for specific key in [`HashMap`].
    Unsigned16(u16),

    /// `signed 32-bit` value stored for specific key in [`HashMap`].
    Signed32(i32),

    /// `unsigned 32-bit` value stored for specific key in [`HashMap`].
    Unsigned32(u32),

    /// `signed 64-bit` value stored for specific key in [`HashMap`].
    Signed64(i64),

    /// `unsigned 64-bit` value stored for specific key in [`HashMap`].
    Unsigned64(u64),

    /// `signed 128-bit` value stored for specific key in [`HashMap`].
    Signed128(i128),

    /// `unsigned 128-bit` value stored for specific key in [`HashMap`].
    Unsigned128(u128),

    /// `32-bit floating point` value stored for specific key in [`HashMap`].
    Float32(f32),

    /// `64-bit floating point` value stored for specific key in [`HashMap`].
    Float64(f64),
}

impl From<String> for ScalarValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for ScalarValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for ScalarValue {
    fn from(value: i8) -> Self {
        Self::Signed8(value)
    }
}

impl From<u8> for ScalarValue {
    fn from(value: u8) -> Self {
        Self::Unsigned8(value)
    }
}

impl From<i16> for ScalarValue {
    fn from(value: i16) -> Self {
        Self::Signed16(value)
    }
}

impl From<u16> for ScalarValue {
    fn from(value: u16) -> Self {
        Self::Unsigned16(value)
    }
}

impl From<i32> for ScalarValue {
    fn from(value: i32) -> Self {
        Self::Signed32(value)
    }
}

impl From<u32> for ScalarValue {
    fn from(value: u32) -> Self {
        Self::Unsigned32(value)
    }
}

impl From<i64> for ScalarValue {
    fn from(value: i64) -> Self {
        Self::Signed64(value)
    }
}

impl From<u64> for ScalarValue {
    fn from(value: u64) -> Self {
        Self::Unsigned64(value)
    }
}

impl From<i128> for ScalarValue {
    fn from(value: i128) -> Self {
        Self::Signed128(value)
    }
}

impl From<u128> for ScalarValue {
    fn from(value: u128) -> Self {
        Self::Unsigned128(value)
    }
}

impl From<f32> for ScalarValue {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for ScalarValue {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}
