//! TODO: Add docs
use std::collections::HashMap;

/// TODO: Add docs
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportResponse {
    /// TODO: Add docs
    pub status: u16,

    /// TODO: Add docs
    pub headers: HashMap<String, String>,

    /// TODO: Add docs
    pub body: Option<Vec<u8>>,
}
