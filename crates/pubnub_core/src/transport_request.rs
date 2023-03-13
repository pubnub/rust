use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub enum TransportMethod {
    #[default]
    Get,
    Post,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportRequest {
    pub path: String,
    pub query_parameters: HashMap<String, String>,
    pub method: TransportMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}
