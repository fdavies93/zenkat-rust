use clap::Parser;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};

#[path = "../common.rs"]
mod common;
use common::node::NodeData;
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
        let path = match tree.get_root().data.clone() {
            NodeData::DocumentData { path, loaded: _ } => path,
            NodeData::DirectoryData { path } => path,
            _ => continue,
        };

        tree_details.push(
            TreeDetail {
                name: tree.name.clone(),
                path,
            },
        );
    }
    return Json(tree_details);
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
                let parser = state.app_config.doc_parser.clone();
                tree.load_all_unloaded_docs(parser).await;
            }
            return Json(Some(
                tree.clone(),
            ));
        }
    }
    return Json(None);
}

async fn query_tree() {}

async fn get_node() {}

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
        let cur_tree = Tree::load(
            name.into(),
            path.into(),
            args.follow_symlinks,
        )
        .await;
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
        .route(
            "/tree",
            get(list_trees),
        )
        .route(
            "/tree/:name",
            get(get_tree),
        )
        .route(
            "/tree/:name/query",
            post(query_tree),
        )
        .route(
            "/tree/:name/:node",
            get(get_node),
        )
        .with_state(state);

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener, app,
    )
    .await
    .unwrap();
}
