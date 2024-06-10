use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq, Hash)]
pub enum ZkRequestType {
    ZkLoad,
    LoadDocs,
    Query,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ZkRequest {
    pub request_type: ZkRequestType,
    pub data: HashMap<String, String>,
}

impl ZkRequest {
    pub fn new(request_type: ZkRequestType) -> ZkRequest {
        let request = ZkRequest {
            request_type,
            data: HashMap::new(),
        };
        return request;
    }
}
