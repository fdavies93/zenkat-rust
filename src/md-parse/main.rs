use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take, take_till, take_until};
use nom::character::complete::{
    alphanumeric0, anychar, char, line_ending, not_line_ending, space0, space1,
};
use nom::character::is_space;
use nom::character::streaming::one_of;
use nom::combinator::{eof, not, opt, rest};
use nom::error::ErrorKind;
use nom::multi::{fold_many0, many0, many1, many1_count, many_m_n, many_till};
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
            alt((
                line_ending,
                eof,
            )),
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

fn little_indent(raw: &str) -> IResult<&str, Vec<char>> {
    many_m_n(
        0,
        3,
        char(' '),
    )(raw)
}

fn atx_header(raw: &str) -> IResult<&str, Tree> {
    let mut parser = tuple((
        little_indent,
        many_m_n(
            1,
            6,
            char('#'),
        ),
        space1,
        many_till(
            anychar,
            line_ending,
        ),
    ));
    let result = parser(raw);
    match result {
        Ok((stream, results)) => {
            let mut node = Node::new(NodeType::HEADER);
            let ack: (
                Vec<char>,
                &str,
            ) = results.3;
            let content: String = ack.0.iter().collect();
            node.data = NodeData::HeaderData {
                text: content,
                level: results.1.len(),
            };
            return Ok((
                stream,
                Tree::new(node),
            ));
        }
        Err(e) => return Err(e),
    }
}

fn thematic_break(raw: &str) -> IResult<&str, Tree> {
    let result = terminated(
        tuple((
            little_indent,
            // commonmark spec is that spaces are allowed
            // between the thematic break characters
            alt((
                tuple((
                    char('-'),
                    space0,
                    char('-'),
                    space0,
                    char('-'),
                    many0(one_of(
                        " \t-",
                    )),
                )),
                tuple((
                    char('_'),
                    space0,
                    char('_'),
                    space0,
                    char('_'),
                    many0(one_of(
                        " \t_",
                    )),
                )),
                tuple((
                    char('*'),
                    space0,
                    char('*'),
                    space0,
                    char('*'),
                    many0(one_of(
                        " \t*",
                    )),
                )),
            )),
            line_ending,
        )),
        line_ending,
    )(raw);

    match result {
        // a thematic break is just a thematic break
        Ok((stream, _)) => {
            let mut node = Node::new(NodeType::THEMATIC_BREAK);
            node.data = NodeData::ThematicBreakData {};
            let tree = Tree::new(node);
            return Ok((
                stream, tree,
            ));
        }
        Err(e) => return Err(e),
    }
}

fn paragraph(raw: &str) -> IResult<&str, Tree> {
    // This is a decent example for transforming a character specification into
    // a unit of meaning
    // Might need an indent level modifier
    let result = tuple((
        little_indent,
        many_till(
            anychar, blank_line,
        ),
    ))(raw);
    match result {
        Ok((stream, results)) => {
            let mut node = Node::new(NodeType::PARAGRAPH);
            let content: String = String::from_iter(results.1 .0);
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
        thematic_break,
        atx_header,
        blank_line,
        paragraph,
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

    let str = read_to_string(args.path.clone()).unwrap();
    let (_, mut tree) = document(str.as_str()).unwrap();
    let root = tree.get_node_mut(tree.root_node.clone());
    root.unwrap().data = NodeData::DocumentData {
        path: args.path.clone(),
        loaded: false,
    };

    let json = to_string(&tree).expect("");

    io::stdout().write_all(json.as_bytes()).expect("");
}
