//! Permission bit flags
//!
//! This module provides bit flags of Access Manager permissions shared between
//! the `access` and `parse_token` features.
//!

/// Resource **read** permissions.
pub(crate) const READ: u8 = 0b0000_0001;
/// Resource **write** permissions.
pub(crate) const WRITE: u8 = 0b0000_0010;
/// Resource **manage** permissions.
pub(crate) const MANAGE: u8 = 0b0000_0100;
/// Resource **delete** permissions.
pub(crate) const DELETE: u8 = 0b0000_1000;
/// Resource **create** permissions.
#[cfg_attr(not(feature = "parse_token"), allow(dead_code))]
pub(crate) const CREATE: u8 = 0b0001_0000;
/// Resource **get** permissions.
pub(crate) const GET: u8 = 0b0010_0000;
/// Resource **update** permissions.
pub(crate) const UPDATE: u8 = 0b0100_0000;
/// Resource **join** permissions.
pub(crate) const JOIN: u8 = 0b1000_0000;
