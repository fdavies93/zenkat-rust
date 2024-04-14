use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    DOCUMENT,
    PARAGRAPH,
    HEADER,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub raw: String,
    pub block_type: NodeType,
    pub blocks: std::vec::Vec<Node>,
    pub data: HashMap<String, String>,
}

impl Node {
    pub fn new(raw: String, block_type: NodeType) -> Self {
        Self {
            raw,
            block_type,
            blocks: vec![],
            data: HashMap::new(),
        }
    }
}
