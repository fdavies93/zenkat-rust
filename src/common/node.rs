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
    LIST_ITEM,
    LIST,
    None, // used in parsing to indicate "consume token but don't email anything"
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ListType {
    UNORDERED_LIST,
    ORDERED_LIST,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum NodeData {
    None,
    HeaderData {
        text: String,
        level: u8,
    },
    DirectoryData {
        path: String,
    },
    DocumentData {
        path: String,
        loaded: bool,
    },
    ParagraphData {
        text: String,
    },
    ListData {
        list_type: ListType,
    },
    ListItemData {
        list_type: ListType,
        text: String,
        indent: usize,
    },
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
