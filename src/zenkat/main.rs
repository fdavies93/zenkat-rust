use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use std::{collections::HashMap, num::NonZeroUsize};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};

#[path = "../common.rs"]
mod common;
use common::node::{Node, NodeData, NodeType};
use common::tree::Tree;

mod app_state;
use app_state::{AppConfig, AppState};

#[derive(Deserialize)]
struct GetTreeParams {
    lod: Option<String>, // Level of Detail
                         // The lowest level type of node to display.
                         // E.g. if it's "document" it will display all documents. If it's "block" it will display only block-level document elements.
}

#[derive(Serialize, Deserialize)]
struct TreeDetail {
    path: String,
    name: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    tree: Vec<String>,

    #[arg(long, default_value = "0")]
    processes: usize,

    #[arg(long, default_value = "")]
    parser: String,

    #[arg(long)]
    follow_symlinks: bool,

    #[arg(long, default_value = "localhost")]
    interface: String,

    #[arg(short, long, default_value = "9001")]
    port: String,
}

async fn list_trees(State(state): State<AppState>) -> Json<Vec<TreeDetail>> {
    let tree_guard = state.trees.lock().await;
    let mut tree_details = vec![];
    for tree in tree_guard.iter() {
        tree_details.push(TreeDetail {
            path: tree.path.clone(),
            name: tree.name.clone(),
        });
    }
    return Json(tree_details);
}

async fn load_document(path: String, parser: String) -> Tree {
    let output = Command::new(parser.as_str())
        .arg(path)
        .output()
        .await
        .expect("");

    let parsed_json = String::from_utf8(output.stdout).expect("");
    let parsed_tree: Tree = serde_json::from_str(parsed_json.as_str()).unwrap();
    return parsed_tree;
}

async fn get_tree(
    Path(name): Path<String>,
    Query(tree_params): Query<GetTreeParams>,
    State(state): State<AppState>,
) -> Json<Option<Tree>> {
    let mut tree_guard = state.trees.lock().await;
    for tree in tree_guard.iter_mut() {
        if tree.name == name {
            let lod = tree_params.lod.unwrap_or("document".into());
            if lod == "block" || lod == "full" {
                // trigger full document load
                let mut set = JoinSet::new();
                let mut path_to_id: HashMap<String, String> = HashMap::new();

                for (_, node) in tree.nodes.iter() {
                    match node.data.clone() {
                        common::node::NodeData::DocumentData { path, loaded } => {
                            if !loaded {
                                path_to_id.insert(path.clone(), node.id.clone());
                                set.spawn(load_document(
                                    path.clone(),
                                    state.app_config.doc_parser.clone(),
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                let mut counted = 0;
                let before = Instant::now();

                while let Some(res) = set.join_next().await {
                    counted += 1;
                    let doc_tree = res.unwrap();
                    let root_id = doc_tree.root_node.clone();
                    let new_root = doc_tree.nodes.get(&root_id).unwrap();
                    let path = doc_tree.path.clone();
                    // copy data to original node, rather than replacing it (so we don't need to recalculate parent links)
                    let og_node_id = path_to_id.get(&path).unwrap();
                    let og_node = tree.nodes.get_mut(og_node_id).unwrap();
                    og_node.children = new_root.children.clone();
                    match og_node.data.clone() {
                        NodeData::DocumentData { path, loaded } => {
                            og_node.data = NodeData::DocumentData { path, loaded: true };
                        }
                        _ => {}
                    }

                    for (node_id, node) in doc_tree.nodes.iter() {
                        if node_id.clone() == root_id {
                            continue;
                        }
                        tree.nodes.insert(node_id.clone(), node.clone());
                    }
                }
                println!("Loaded {} documents in {:.4?}.", counted, before.elapsed());
            }
            return Json(Some(tree.clone()));
        }
    }
    return Json(None);
}

async fn put_tree() {}

async fn query_tree() {}

async fn get_node() {}

// design point: PUT /tree might be unnecessary as if this is a RESTful
// API then state of cache shouldn't really be directly controllable
// on such a granular level
// for debugging / early dev it's probably ok however
#[tokio::main]
async fn main() {
    let mut processes = thread::available_parallelism().expect("");
    let mut parser = String::from("target/debug/md-parse");

    let args = Args::parse();

    if args.processes > 0 {
        processes = NonZeroUsize::new(args.processes).expect("");
    }

    if args.parser.len() > 0 {
        parser = args.parser;
    }

    let mut trees = vec![];
    for tree_arg in args.tree {
        let (name, path) = tree_arg.split_once(":").unwrap();
        let cur_tree = Tree::load(name.into(), path.into(), args.follow_symlinks).await;
        match cur_tree {
            Some(tree) => trees.push(tree),
            None => {}
        }
    }

    let state = AppState {
        trees: Arc::new(Mutex::new(trees.to_owned())),
        app_config: AppConfig {
            follow_symlinks: args.follow_symlinks,
            doc_parser: parser,
            processes,
        },
    };

    let app = Router::new()
        .route("/tree", get(list_trees))
        .route("/tree", put(put_tree)) // unclear if this one is necessary
        .route("/tree/:name", get(get_tree))
        .route("/tree/:name/query", post(query_tree))
        .route("/tree/:name/:node", get(get_node))
        .with_state(state);

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
