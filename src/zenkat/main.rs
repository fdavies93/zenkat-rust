use clap::Parser;
use std::sync::Arc;
use std::thread;
use std::{borrow::Borrow, num::NonZeroUsize};
use tokio::sync::Mutex;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};

#[path = "../common.rs"]
mod common;

mod app_state;
use app_state::{AppConfig, AppState};

struct Node {}

struct Tree {
    path: String,
    root_node: Node,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    trees: Vec<String>,

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

async fn list_trees(State(state): State<AppState>) {
    let mut data = state.trees.lock_owned();
}

async fn get_tree() {}

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

    let state = AppState {
        trees: Arc::new(Mutex::new(vec![])),
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
        .route("/tree/:name/node", get(get_node))
        .with_state(state);

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
