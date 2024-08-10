use async_process::Output;
use clap::Parser;
use serde_json::to_string;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self, Write};

#[path = "../common.rs"]
mod common;
use common::node::{Node, NodeData, NodeType};
use common::tree::Tree;

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn parse_atx_header(raw: String) -> Option<(String, Node)> {
    let mut unconsumed = raw;
    let mut buffer = String::new();

    // parse leading # signs
    loop {
        if unconsumed.is_empty() {
            break;
        }

        let first_char = unconsumed.chars().next().unwrap();

        let (prefix, suffix) = unconsumed.split_at(first_char.len_utf8());

        if prefix == "#" {
            buffer.push_str(prefix);
            unconsumed = String::from(suffix);
        } else if prefix == " " {
            unconsumed = String::from(suffix);
            break;
        } else {
            // #abc is not a valid ATX header; it's a tag
            return None;
        }
    }

    if buffer.len() < 1 {
        return None;
    }

    let rank = buffer.len();

    buffer.clear();

    // could add a section here for handling whitespace between ATX tag and header text

    // parse rest of line
    loop {
        if unconsumed.is_empty() {
            break;
        }

        let first_char = unconsumed.chars().next().unwrap();

        let (prefix, suffix) = unconsumed.split_at(first_char.len_utf8());

        if prefix == "\n" {
            break;
        }

        buffer.push_str(prefix);
        unconsumed = String::from(suffix);
    }

    // ## is not a valid header; it's empty
    if buffer.len() == 0 {
        return None;
    }

    let mut node = Node::new(NodeType::HEADER);
    node.data = NodeData::HeaderData {
        text: buffer,
        level: u8::try_from(rank).unwrap(),
        // if we have more than 255 # something is horribly wrong
    };

    return Some((unconsumed, node));
}

fn parse_paragraph(raw: String) -> Option<(String, Node)> {
    let mut unconsumed = raw;
    let mut buffer = String::new();
    let mut output_buffer = String::new();
    loop {
        if unconsumed.is_empty() {
            break;
        }

        let first_char = unconsumed.chars().next().unwrap();

        let (prefix, suffix) = unconsumed.split_at(first_char.len_utf8());

        if prefix == "\n" {
            buffer.push_str(prefix);
        } else {
            if buffer.len() >= 2 {
                break;
            }
            output_buffer.push_str(buffer.as_str());
            buffer.clear();
            output_buffer.push_str(prefix);
        }
        unconsumed = String::from(suffix);
    }

    let mut output = Node::new(NodeType::PARAGRAPH);
    output.data = NodeData::ParagraphData {
        text: output_buffer,
    };

    return Some((unconsumed, output));
}

fn parse_document(path: &str) -> Tree {
    let parse_fns: Vec<&dyn Fn(String) -> Option<(String, Node)>> =
        vec![&parse_atx_header, &parse_paragraph];
    let raw = read_to_string(path).expect("");
    let mut root = Node::new(NodeType::DOCUMENT);

    root.data = NodeData::DocumentData {
        path: String::from(path),
        loaded: true,
    };
    let mut output = Tree::new();
    output.path = path.into();
    output.root_node = root.id.clone();

    let mut to_parse = String::from(&raw);

    loop {
        if to_parse.is_empty() {
            break;
        }

        for f in &parse_fns {
            let result = f(to_parse.clone());

            if result.is_some() {
                let (remaining, node) = result.unwrap();
                to_parse = remaining;

                root.children.push(node.id.clone());
                output.nodes.insert(node.id.clone(), node);
            }
        }
    }
    output.nodes.insert(root.id.clone(), root);

    return output;
}

fn main() {
    let args = Args::parse();

    let tree = parse_document(&args.path);

    let json = to_string(&tree).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
