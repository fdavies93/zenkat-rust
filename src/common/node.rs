use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum NodeType {
    DIRECTORY,
    DOCUMENT,
    PARAGRAPH,
    HEADER,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub raw: String,
    pub block_type: NodeType,
    pub blocks: std::vec::Vec<Node>,
    pub data: HashMap<String, String>,
}

impl Node {
    pub fn new(raw: String, block_type: NodeType) -> Self {
        let id = Uuid::new_v4();
        let id_as_str = id.to_string();
        Self {
            id: id_as_str,
            raw,
            block_type,
            blocks: vec![],
            data: HashMap::new(),
        }
    }

    pub fn type_as_string(&self) -> &str {
        match self.block_type {
            NodeType::DIRECTORY => "directory",
            NodeType::HEADER => "header",
            NodeType::DOCUMENT => "document",
            NodeType::PARAGRAPH => "paragraph",
        }
    }
}
