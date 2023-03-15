use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum TransportMethod {
    #[default]
    Get,
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

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportRequest {
    pub path: String,
    pub query_parameters: HashMap<String, String>,
    pub method: TransportMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}
