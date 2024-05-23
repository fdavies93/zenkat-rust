use clap::Parser;
use std::num::NonZeroUsize;
use std::thread;

#[path = "../common.rs"]
mod common;

mod tree_store;
use tree_store::TreeStore;

mod query_parser;

#[derive(Parser, Debug)]
struct Args {
    paths: Option<Vec<String>>,

    #[arg(short, long, default_value = "0")]
    processes: usize,

    #[arg(long, default_value = "")]
    parser: String,

    #[arg(short, long)]
    query: Option<String>,

    #[arg(long)]
    trees: bool,

    #[arg(long)]
    load_all: bool,
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

    let paths = args.paths.unwrap_or(vec![]);

    let mut store = TreeStore::load(paths, true);

    if args.load_all {
        let docs = store.get_all_documents_mut();
        TreeStore::hydrate_docs(docs, processes.into(), &parser).await;
    }
    if args.query.is_some() {
        let collected = store.query(&args.query.unwrap());
        let as_json = serde_json::to_string_pretty(&collected).unwrap();
        println!("{}", as_json);
    }
    if args.trees {
        for tree in store.get_trees() {
            println!("{}", tree.data.get("path").unwrap());
        }
    }
}
