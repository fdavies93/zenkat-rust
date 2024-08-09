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
    None,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum NodeDeltaOperation {
    APPEND_AS_CHILD,
    DROP,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum NodeData {
    None,
    HeaderData { text: String, level: u8 },
    DirectoryData { path: String },
    DocumentData { path: String },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NodeDelta {
    pub source_id: String,
    pub op: NodeDeltaOperation,
    pub payload: Node,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub children: Vec<String>,
    pub data: NodeData,
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        let id = Uuid::new_v4();
        let id_as_str = id.to_string();
        Self {
            id: id_as_str,
            node_type,
            children: vec![],
            data: NodeData::None,
        }
    }
}
