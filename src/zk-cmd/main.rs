use axum::handler::Handler;
use clap::{Parser, Subcommand};
use reqwest;
use tokio;

#[path = "../common.rs"]
mod common;
use common::tree::Tree;

use crate::common::node::NodeData;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,

    #[arg(long, default_value = "http")]
    protocol: String,

    #[arg(long, default_value = "9001")]
    port: String,

    #[arg(long, default_value = "localhost")]
    host: String,
}

#[derive(Debug, Subcommand)]
enum Command {
    Tree { name: String },
}

fn visualise_tree(tree: &Tree, cur_node_id: String, cur_depth: usize) {
    let cur_node = tree.nodes.get(&cur_node_id).unwrap();
    let indent = "  ".repeat(cur_depth);

    let mut content: String = String::new();

    match cur_node.data.clone() {
        NodeData::DirectoryData { path } => {
            content = path.clone();
        }
        NodeData::DocumentData { path, loaded: _ } => {
            content = path.clone();
        }
        NodeData::HeaderData { text, level } => content = format!("<h{}> {}", level, text),
        NodeData::ParagraphData { text: _ } => content = "<p>".into(),
        _ => {}
    }

    println!("{}{}", indent, content);

    for child in cur_node.children.iter() {
        visualise_tree(tree, child.clone(), cur_depth + 1);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let server_uri = vec![
        args.protocol,
        "://".into(),
        args.host,
        ":".into(),
        args.port,
    ]
    .join("");

    match &args.cmd {
        Command::Tree { name } => {
            let uri = vec![server_uri, "tree".into(), name.into()].join("/");
            let uri_params = vec![uri.clone(), "lod=full".into()].join("?");

            let tree: Tree = client
                .get(uri_params)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            visualise_tree(&tree, tree.root_node.clone(), 0);
        }
    };
}
