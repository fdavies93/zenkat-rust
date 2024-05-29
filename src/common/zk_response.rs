use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ZkResponseType {
    ZkLoad,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ZkResponse {
    pub data: HashMap<String, String>,
}

impl ZkResponse {
    pub fn new() -> ZkResponse {
        let response = ZkResponse {
            data: HashMap::new(),
        };
        return response;
    }
}
