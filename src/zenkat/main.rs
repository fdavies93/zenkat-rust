use clap::Parser;
use std::collections::{vec_deque, HashMap, VecDeque};
use std::path::Path;
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
use common::node::{Node, NodeData, NodeType};

mod app_state;
use app_state::{AppConfig, AppState};

#[derive(Debug, Clone)]
struct Tree {
    path: String,
    root_node: String,
    nodes: HashMap<String, Node>,
}

impl Tree {
    pub fn new() -> Tree {
        return Tree {
            path: String::new(),
            root_node: String::new(),
            nodes: HashMap::new(),
        };
    }

    pub fn load(path: String, traverse_symbolic: bool) -> Option<Tree> {
        let mut tree: Tree = Tree::new();
        tree.path = path.clone();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut parents: HashMap<String, String> = HashMap::new();

        while queue.len() > 0 {
            let path_str = queue.pop_front()?;
            let cur_path = Path::new(&path_str);
            let mut cur_node: Node = Node::new(NodeType::None);
            let cur_node_id: String = cur_node.id.clone();
            if !cur_path.exists() {
                continue;
            } else if cur_path.is_symlink() && !traverse_symbolic {
                continue;
            } else if cur_path.is_file() {
                if cur_path.extension()? != "md" {
                    return None;
                }
                cur_node.node_type = NodeType::DOCUMENT;
                cur_node.data = NodeData::DocumentData {
                    path: cur_path.to_str()?.into(),
                };
                tree.nodes.insert(cur_node_id.clone(), cur_node);
            } else if cur_path.is_dir() {
                cur_node.node_type = NodeType::DIRECTORY;
                cur_node.data = NodeData::DirectoryData {
                    path: cur_path.to_str()?.into(),
                };

                let children = cur_path.read_dir().ok()?;
                for child in children {
                    let c_path = child.ok()?.path();
                    let c_path_str: String = c_path.to_str()?.into();
                    queue.push_back(c_path_str.clone());
                    parents.insert(c_path_str, cur_node_id.clone());
                }
                tree.nodes.insert(cur_node_id.clone(), cur_node);
            }
            // link the parents to the children by ID using hash table
            let parent_id = parents.get(&path_str);
            match parent_id {
                Some(_) => {
                    let parent = tree.nodes.get_mut(parent_id.unwrap())?;
                    parent.children.push(cur_node_id.clone());
                }
                None => tree.root_node = cur_node_id.clone(),
            }
        }
        return Some(tree);
    }
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

async fn list_trees(State(state): State<AppState>) -> Json<Vec<String>> {
    let tree_guard = state.trees.lock().await;
    let mut tree_details = vec![];
    for tree in tree_guard.iter() {
        tree_details.push(tree.path.clone());
    }
    return Json(tree_details);
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

    let mut trees = vec![];
    for path in args.tree {
        println!("{}", path);
        trees.push(Tree {
            path,
            root_node: "".to_string(),
            nodes: HashMap::new(),
        });
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
        .route("/tree/:name/node", get(get_node))
        .with_state(state);

    let addr = vec![args.interface, ":".into(), args.port.into()].join("");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
