use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct TransportResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>
}