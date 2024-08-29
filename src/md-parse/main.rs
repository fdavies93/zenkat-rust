use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::{is_a, take_until};
use nom::character::complete::char;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::tuple;
use serde_json::to_string;
use std::io::{self, Write};

#[path = "../common.rs"]
mod common;
use common::node::{ListType, Node, NodeData, NodeType};
use common::tree::Tree;

use nom::IResult;
#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn paragraph(raw: &str) -> IResult<&str, Node> {
    // This is a decent example for transforming a character specification into
    // a unit of meaning
    let result = tuple((opt(char('\n')), take_until("\n\n")))(raw);
    match result {
        Ok((stream, results)) => {
            let mut node = Node::new(NodeType::PARAGRAPH);
            let content = results.1;
            node.data = NodeData::ParagraphData {
                text: content.into(),
            };
            return Ok((stream, node));
        }
        Err(e) => return Err(e),
    }
}

fn block(raw: &str) -> IResult<&str, Node> {
    return alt((paragraph,))(raw);
}

fn document(path: &String) -> Tree {
    many0(block)(path);
    return Tree::new(Node::new(NodeType::DOCUMENT));
}

fn main() {
    let args = Args::parse();

    let tree = document(&args.path);

    let json = to_string(&tree).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
