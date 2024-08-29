use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take, take_till, take_until};
use nom::character::complete::{
    alphanumeric0, anychar, char, line_ending, not_line_ending, space0,
};
use nom::combinator::{eof, not, opt, rest};
use nom::multi::{fold_many0, many0, many1, many_till};
use nom::sequence::{terminated, tuple};
use serde_json::to_string;
use std::fs::{self, read_to_string};
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

fn blank_line(raw: &str) -> IResult<&str, Tree> {
    let result = tuple((
        line_ending,
        terminated(
            space0,
            line_ending,
        ),
    ))(raw);
    match result {
        Ok((stream, results)) => {
            let node = Node::new(NodeType::None);
            let tree = Tree::new(node);
            return Ok((
                stream, tree,
            ));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

fn paragraph(raw: &str) -> IResult<&str, Tree> {
    // This is a decent example for transforming a character specification into
    // a unit of meaning
    // Might need an indent level modifier
    let result = many_till(
        anychar, blank_line,
    )(raw);
    match result {
        Ok((stream, results)) => {
            let mut node = Node::new(NodeType::PARAGRAPH);
            let content: String = String::from_iter(results.0);
            node.data = NodeData::ParagraphData { text: content };
            return Ok((
                stream,
                Tree::new(node),
            ));
        }
        Err(e) => return Err(e),
    }
}

fn block(raw: &str) -> IResult<&str, Tree> {
    return alt((
        blank_line, paragraph,
    ))(raw);
}

fn document(raw: &str) -> IResult<&str, Tree> {
    let blocks = many0(block)(raw);
    match blocks {
        Ok((stream, results)) => {
            let root = Node::new(NodeType::DOCUMENT);
            let mut tree = Tree::new(root);
            for block in results {
                tree.insert_child_under(
                    block,
                    tree.root_node.clone(),
                );
            }
            return Ok((
                stream, tree,
            ));
        }
        Err(e) => return Err(e),
    }
}

fn main() {
    let args = Args::parse();

    let str = read_to_string(args.path).unwrap();
    let tree = document(str.as_str()).unwrap();

    let json = to_string(&tree).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
