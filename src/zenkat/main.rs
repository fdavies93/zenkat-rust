use axum::extract::State;
use clap::Parser;
use std::pin::Pin;
use std::thread;
use std::{num::NonZeroUsize, sync::Arc};

#[path = "../common.rs"]
mod common;

mod tree_store;
use tree_store::TreeStore;

use axum::{
    debug_handler,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use common::zk_request::{ZkRequest, ZkRequestType};
use common::zk_response::ZkResponse;
use std::future::Future;

mod query_parser;
use query_parser::QueryParser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    zk: Vec<String>,

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

async fn handle_request(Json(payload): Json<ZkRequest>) -> (StatusCode, Json<ZkResponse>) {
    let res = ZkResponse::new();

    println!("{:?}", payload);

    return (StatusCode::OK, Json(res));
}

#[debug_handler]
async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ZkRequest>,
) -> (StatusCode, Json<ZkResponse>) {
    let res = ZkResponse::new();
    state.parser.trigger(&payload);
    println!("{:?}", payload);
    return (StatusCode::OK, Json(res));
}

struct AppState {
    parser: QueryParser,
    store: TreeStore,
}

fn load_zk() {
    println!("Fake ZK load!");
}

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

    let store = TreeStore::load(args.zk, args.follow_symlinks);

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");

    println!("Starting Zenkat HTTP server on {}", addr);

    let mut query_parser = QueryParser::new();
    query_parser.bind(ZkRequestType::ZkLoad, load_zk);

    let state = Arc::new(AppState {
        parser: query_parser,
        store: store,
    });

    // setup web server with Axum
    let app = Router::new().route("/", post(handle)).with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // if args.load_all {
    //     let docs = store.get_all_documents_mut();
    //     TreeStore::hydrate_docs(docs, processes.into(), &parser).await;
    // }
    // if args.query.is_some() {
    //     let collected = store.query(&args.query.unwrap());
    //     let as_json = serde_json::to_string_pretty(&collected).unwrap();
    //     println!("{}", as_json);
    // }
    // if args.trees {
    //     for tree in store.get_trees() {
    //         println!("{}", tree.data.get("path").unwrap());
    //     }
    // }
}
