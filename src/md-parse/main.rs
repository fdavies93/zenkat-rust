use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::fs::read_to_string;
use std::io::{self, Write};

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    DOCUMENT,
    PARAGRAPH,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub raw: String,
    pub block_type: NodeType,
    pub blocks: std::vec::Vec<Node>,
}

impl Node {
    pub fn new(raw: String, block_type: NodeType) -> Self {
        Self {
            raw,
            block_type,
            blocks: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct MDocument {
    pub path: String,
    pub raw: String,
    pub blocks: std::vec::Vec<Node>,
}

impl MDocument {
    pub fn new(path: String, raw: String) -> Self {
        Self {
            path,
            raw,
            blocks: vec![],
        }
    }
}

fn parse_paragraph(raw: String) -> Option<(String, Node)> {
    let mut unconsumed = raw;
    let mut buffer = String::new();
    let mut output_buffer = String::new();
    loop {
        if buffer == "\n\n" || unconsumed.is_empty() {
            break;
        }

        let (prefix, suffix) = unconsumed.split_at(1);
        if prefix == "\n" {
            buffer.push_str(prefix);
        } else {
            output_buffer.push_str(buffer.as_str());
            buffer.clear();
            output_buffer.push_str(prefix);
        }
        unconsumed = String::from(suffix);
    }

    if output_buffer.is_empty() {
        // might this introduce an edge case error? yes!
        return None;
    }

    let output = Node::new(output_buffer, NodeType::PARAGRAPH);

    return Some((unconsumed, output));
}

fn parse_document(path: &str) -> MDocument {
    let raw = read_to_string(path).expect("");
    let mut output = MDocument::new(String::from(path), raw.clone());
    let mut to_parse = String::from(&raw);

    loop {
        // println!("{}", to_parse.len());
        if to_parse.is_empty() {
            break;
        }
        let result = parse_paragraph(to_parse.clone());
        if result.is_some() {
            let (remaining, node) = result.unwrap();
            to_parse = remaining;
            output.blocks.push(node);
        }
    }

    return output;
}

fn main() {
    let args = Args::parse();

    let document = parse_document(&args.path);

    let json = to_string(&document).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
