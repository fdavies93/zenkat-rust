use axum::extract::State;
use clap::Parser;
use std::sync::RwLock;
use std::thread;
use std::{num::NonZeroUsize, pin::Pin, sync::Arc};

#[path = "../common.rs"]
mod common;

mod tree_store;
use tree_store::TreeStore;

mod app_state;
use app_state::{AppConfig, AppState};

use axum::{
    debug_handler,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use common::zk_request::{ZkRequest, ZkRequestType};
use common::zk_response::{ZkResponse, ZkResponseType};

mod query_parser;
use query_parser::{QueryParser, ZkResponseType};

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

#[debug_handler]
async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ZkRequest>,
) -> (StatusCode, Json<ZkResponse>) {
    let res = ZkResponse::new();

    let parser = &state.parser;

    // this causes an indefinite hang
    // since it needs to get a read for parser and a write for store
    parser.trigger(&payload, &state);

    println!("{:?}", payload);
    return (StatusCode::OK, Json(res));
}

fn load_zk(request: &ZkRequest, state: &Arc<AppState>) -> ZkResponseType {
    let path = request.data.get("path").unwrap();

    let mut store = state.store.write().unwrap();

    store.load(vec![path.clone()], true);

    println!("Loading tree at {}.", path);

    return Box::pin(async { Result::Ok(ZkResponse::new()) });

    // return Pin::new(Box::pin(async { ZkResponse::new() }));
    //return Pin::new(Result::Ok(ZkResponse::new()));
}

fn query(request: &ZkRequest, state: &Arc<AppState>) -> Result<ZkResponse, &'static str> {
    let mut store = state.store.write().unwrap();

    let results = store.query(request.data.get("query").unwrap());

    // could make ZkResponse a trait for better serialisation
    let mut res = ZkResponse::new();
    res.data
        .insert("nodes".into(), serde_json::to_string(&results).unwrap());

    return Result::Ok(res);
}

// take the appState
// extract a reference to store
// wait for the docs to be hydrated
// update the docs within asynchronously

fn load_docs(request: &ZkRequest, state: Arc<AppState>) -> ZkResponseType {
    let mut store = state.store.write().unwrap();
    return Box::pin(async move {
        TreeStore::hydrate_docs(
            // this line is the problem
            // it modifies the documents that were passed in - asynchronously
            // therefore it's impossible for the borrow checker to know what's happening - this is inherently unsafe
            // instead we should try preparing changes to in-memory docs in a async-safe queue
            store.get_all_documents_mut(),
            &state.app_config.processes,
            &state.app_config.doc_parser,
        )
        .await;

        return Result::Ok(ZkResponse::new());
    });
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

    let store = TreeStore::new();

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");

    println!("Starting Zenkat HTTP server on {}", addr);

    let config = AppConfig {
        follow_symlinks: args.follow_symlinks,
        doc_parser: parser,
        processes,
    };

    let mut query_parser = QueryParser::new();
    query_parser.bind(ZkRequestType::ZkLoad, load_zk);
    query_parser.bind(ZkRequestType::LoadDocs, load_docs);
    query_parser.bind(ZkRequestType::Query, query);

    // the server config should also be held in here, immutably
    let state = Arc::new(AppState {
        parser: query_parser,
        store: RwLock::new(store),
        app_config: config,
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
