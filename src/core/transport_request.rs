//! TODO: Add documentation
use std::{collections::HashMap, fmt::Display};

/// TODO: Add docs
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum TransportMethod {
    /// TODO: Add docs
    #[default]
    Get,

    /// TODO: Add docs
    Post,
}

impl Display for TransportMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransportMethod::Get => "GET",
                TransportMethod::Post => "POST",
            }
        )
    }
}

/// TODO: Add docs
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportRequest {
    /// TODO: Add docs
    pub path: String,

    /// TODO: Add docs
    pub query_parameters: HashMap<String, String>,

    /// TODO: Add docs
    pub method: TransportMethod,

    /// TODO: Add docs
    pub headers: HashMap<String, String>,

    /// TODO: Add docs
    pub body: Option<Vec<u8>>,
}
