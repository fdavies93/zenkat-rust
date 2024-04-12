use clap::Parser;
use nom::IResult;
use std::io::{self, Write};
use std::{thread, time, vec};

#[derive(Debug, PartialEq)]
pub enum BlockType {
    PARAGRAPH,
}

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub raw: String,
    pub block_type: BlockType,
}

#[derive(Debug, PartialEq)]
pub struct MDocument {
    pub blocks: std::vec::Vec<Block>,
}

fn main() {
    let args = Args::parse();
    let output = format!("{{ \"path\": \"{}\" }}", &args.path);
    io::stdout().write_all(output.as_bytes()).unwrap();
}
