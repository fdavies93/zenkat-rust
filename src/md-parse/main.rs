use async_process::Output;
use clap::Parser;
use serde_json::to_string;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self, Write};

#[path = "../common.rs"]
mod common;
use common::node::{ListType, Node, NodeData, NodeType};
use common::tree::Tree;

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn parse_atx_header(raw: String) -> Option<(String, Tree)> {
    let mut unconsumed = raw;
    let mut buffer = String::new();

    // there should be a maximum of one leading \n
    loop {
        if unconsumed.is_empty() {
            break;
        }

        let first_char = unconsumed.chars().next().unwrap();

        let (prefix, suffix) = unconsumed.split_at(first_char.len_utf8());

        if prefix == "\n" {
            buffer.push_str(prefix);
            unconsumed = String::from(suffix);
        } else {
            break;
        }

        if buffer.len() >= 2 {
            return None;
        }
    }

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

    return Some((unconsumed, Tree::new(node)));
}

fn parse_paragraph(raw: String) -> Option<(String, Tree)> {
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

    return Some((unconsumed, Tree::new(output)));
}

fn parse_blank_line(raw: String) -> Option<(String, Tree)> {
    // just emit a token
    let mut unconsumed = raw;

    let next_char = match unconsumed.chars().next() {
        Some(out) => out,
        None => return None,
    };

    if next_char != '\n' {
        return None;
    }

    let (_, suffix) = unconsumed.split_at(next_char.len_utf8());
    unconsumed = String::from(suffix);

    loop {
        if unconsumed.is_empty() {
            break;
        }

        let next_char = unconsumed.chars().next().unwrap();

        if next_char == ' ' || next_char == '\t' {
            // i.e. ignore
            let (_, suffix) = unconsumed.split_at(next_char.len_utf8());
            unconsumed = String::from(suffix);
        } else if next_char == '\n' {
            break;
        } else {
            return None;
        }
    }

    let output = Node::new(NodeType::None);
    return Some((unconsumed, Tree::new(output)));
}

fn parse_uint(raw: String) -> Option<(String, u64)> {
    let mut unconsumed = raw;
    let mut buffer: String = String::new();

    loop {
        if unconsumed.is_empty() {
            break;
        }
        let next_char = unconsumed.chars().next().unwrap();

        if !next_char.is_digit(10) {
            break;
        }

        let (_, suffix) = unconsumed.split_at(next_char.len_utf8());

        buffer.push(next_char);
        unconsumed = suffix.into();
    }

    if buffer.len() == 0 {
        return None;
    }

    let integer: u64 = buffer.parse().unwrap();

    return Some((unconsumed, integer));
}

fn parse_list_item_content(raw: String) -> (String, String) {
    return (String::new(), String::new());
}

fn one_of(raw: String, options: Vec<char>) -> Option<(String, char)> {
    let unconsumed = raw;
    if unconsumed.is_empty() {
        return None;
    }
    let next_char = unconsumed.chars().next().unwrap();
    for option in options {
        if next_char == option {
            let (_, suffix) = unconsumed.split_at(next_char.len_utf8());
            return Some((suffix.into(), next_char));
        }
    }
    return None;
}

fn one(chr: char) -> Box<dyn Fn(String) -> Option<String>> {
    Box::new(move |raw: String| -> Option<String> {
        if raw.is_empty() {
            return None;
        }
        let next_char = raw.chars().next().unwrap();
        if next_char != chr {
            return None;
        }
        let (_, suffix) = raw.split_at(next_char.len_utf8());
        return Some(suffix.into());
    })
}

fn any(chrs: &'static str) -> Box<dyn Fn(String) -> Option<(String, char)>> {
    Box::new(move |raw: String| -> Option<(String, char)> {
        if raw.is_empty() {
            return None;
        }

        let next_char = raw.chars().next().unwrap();
        for chr in chrs.chars() {
            if next_char == chr {
                let (_, suffix) = raw.split_at(next_char.len_utf8());
                return Some((suffix.into(), chr));
            }
        }
        return None;
    })
}

fn many0<T>(
    input_fn: Box<dyn Fn(String) -> Option<(String, T)>>,
) -> Box<dyn Fn(String) -> Option<(String, Vec<T>)>> {
    Box::new(move |raw: String| -> Option<(String, Vec<T>)> {
        let output: Vec<T> = vec![];
        loop {
            let result = input_fn(raw.clone());
        }
        return Some((raw, output));
    })
}

fn parse_ordered_list_item_bullet(raw: String) -> Option<(String, u64)> {
    let mut unconsumed = raw;
    let li_num: u64;

    match parse_uint(unconsumed) {
        Some((stream, number)) => {
            unconsumed = stream;
            li_num = number;
        }
        None => return None,
    }

    match one_of(unconsumed, vec!['.', ')']) {
        Some((stream, _delimiter)) => {
            unconsumed = stream;
        }
        None => return None,
    }

    return Some((unconsumed, li_num));
}

fn parse_unordered_list_item_bullet(raw: String) -> Option<(String, char)> {
    any("-*+")(raw)
}

fn parse_list_item_bullet(raw: String) -> Option<(String, ListType)> {
    let ordered = parse_ordered_list_item_bullet(raw.clone());
    match ordered {
        Some((unconsumed, _num)) => return Some((unconsumed, ListType::ORDERED_LIST)),
        None => (),
    }

    let unordered = parse_unordered_list_item_bullet(raw.clone());
    match unordered {
        Some(unconsumed) => return Some((unconsumed, ListType::UNORDERED_LIST)),
        None => return None,
    }
}

fn parse_list_item(raw: String) -> Option<(String, Tree)> {
    let mut unconsumed = raw;
    let mut buffer = String::new();
    // skip leading whitespace
    loop {
        if unconsumed.is_empty() {
            break;
        }
        let next_char = unconsumed.chars().next().unwrap();
        if next_char == ' ' || next_char == '\t' {
            let (prefix, suffix) = unconsumed.split_at(next_char.len_utf8());
            buffer.push_str(prefix);
            unconsumed = String::from(suffix);
        }
    }
    let indent = buffer.len();
    // get which type of list item it is
    let list_type = match parse_list_item_bullet(unconsumed.clone()) {
        Some((out, lt)) => {
            unconsumed = out;
            lt
        }
        None => return None,
    };

    // this is a little 'interesting' due to the presence of
    // loose lists in the markdown spec
    let (unconsumed, content) = parse_list_item_content(unconsumed);

    let mut node = Node::new(NodeType::LIST_ITEM);
    let data = NodeData::ListItemData {
        list_type,
        text: content,
        indent,
    };

    node.data = data;
    let tree = Tree::new(node);
    return Some((unconsumed, tree));
}

fn parse_list(raw: String) -> Option<(String, Tree)> {
    let mut working_tree = Tree::new(Node::new(NodeType::LIST));
    let mut unconsumed = raw.clone();
    let mut have_items: bool = false;

    loop {
        let option = parse_list_item(unconsumed.clone());
        match option {
            Some((new_raw, output)) => {
                unconsumed = new_raw.clone();
                working_tree.insert_child_under(output, working_tree.get_root_id());
                have_items = true;
            }
            None => {
                break;
            }
        }
    }

    if !have_items {
        return None;
    }

    return Some((unconsumed, working_tree));
}

fn parse_document(path: &str) -> Tree {
    let parse_fns: Vec<&dyn Fn(String) -> Option<(String, Tree)>> = vec![
        &parse_atx_header,
        &parse_list,
        &parse_blank_line,
        &parse_paragraph,
    ];
    let raw = read_to_string(path).expect("");
    let mut root = Node::new(NodeType::DOCUMENT);

    let root_id = root.id.clone();
    root.data = NodeData::DocumentData {
        path: String::from(path),
        loaded: true,
    };
    let mut output = Tree::new(root);

    let mut to_parse = String::from(&raw);

    loop {
        if to_parse.is_empty() {
            break;
        }

        for f in &parse_fns {
            let result = f(to_parse.clone());
            if result.is_some() {
                let (remaining, subtree) = result.unwrap();
                to_parse = remaining;

                let child_root = subtree.get_root();

                if child_root.node_type == NodeType::None {
                    break;
                }

                output.insert_child_under(subtree, root_id.clone());
            }
        }
    }

    return output;
}

fn main() {
    let args = Args::parse();

    let tree = parse_document(&args.path);

    let json = to_string(&tree).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
