use async_process::Child;
use clap::Parser;
use serde_json::{from_str, to_string_pretty};
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::thread;
use tokio::process::Command;

#[path = "../common/node.rs"]
mod common;
use common::{Node, NodeType};

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

#[derive(Debug)]
struct TreeStore {
    trees: Vec<Node>,
}

async fn parse_at_paths(paths: Vec<PathBuf>, processes: usize, parser: String) -> Vec<Node> {
    let mut parsed: Vec<Node> = vec![];
    let mut children: VecDeque<_> = VecDeque::new();
    let mut remaining_paths: Vec<PathBuf> = paths.to_vec();

    loop {
        if remaining_paths.len() == 0 && children.len() == 0 {
            break;
        }
        loop {
            if children.len() == processes || remaining_paths.len() == 0 {
                break;
            }
            let next_path = remaining_paths.pop().expect("");
            let child = Command::new(parser.as_str()).arg(&next_path).output();
            children.push_back(child);
        }
        let output = children.pop_front().expect("").await.expect("");
        let string = String::from_utf8(output.stdout).expect("");
        let parsed_string: Node = serde_json::from_str(string.as_str()).expect("");
        parsed.push(parsed_string);
    }
    return parsed;
}

// old process:
// - walk the directories, return paths of .md files as list
// - parse each .md file asynchronously
// - finally, return list of all parsed entries

// new process:
// - walk the directories, return Nodes corresponding to the root of each path
// - walk Node tree and add each DOCUMENT to the list to parse
// - parse them asynchronously and add their nodes to the tree
// - this could later become a JIT process

impl TreeStore {
    fn load_tree(path: &Path, traverse_symbolic: bool) -> Option<Node> {
        if path.is_symlink() && !traverse_symbolic {
            return None;
        }
        let mut output: Node = Node::new(String::new(), NodeType::DOCUMENT);

        if path.is_dir() {
            output.block_type = NodeType::DIRECTORY;
            let children = path.read_dir().ok()?;
            let mut child_nodes = vec![];

            for child in children {
                let c_path = child.ok()?;
                let node_choice = TreeStore::load_tree(c_path.path().as_path(), traverse_symbolic);

                if node_choice.is_some() {
                    child_nodes.push(node_choice.unwrap());
                }
            }

            output.blocks = child_nodes;
        }

        if path.is_file() && path.extension().unwrap() != "md" {
            return None;
        }

        let path_str = String::from(path.as_os_str().to_str().unwrap());
        output.data.insert(String::from("path"), path_str);

        return Some(output);
    }

    fn load(paths: Vec<String>, traverse_symbolic: bool) -> TreeStore {
        let mut store: TreeStore = TreeStore { trees: vec![] };

        for path_str in paths {
            let path = Path::new(&path_str);

            if !path.try_exists().is_ok_and(|x| x) {
                println!(
                    "Couldn't access path at {} when loading tree store.",
                    path_str.as_str()
                );
                continue;
            }

            let tree_option = TreeStore::load_tree(path, traverse_symbolic);

            if tree_option.is_some() {
                store.trees.push(tree_option.unwrap());
            }
        }

        return store;
    }

    // likely needs to work on references
    fn get_all_documents_mut(&mut self) -> Vec<&mut Node> {
        let mut docs: Vec<&mut Node> = vec![];

        let mut remaining: VecDeque<&mut Node> = VecDeque::new();
        for root in (self.trees).iter_mut() {
            remaining.push_back(root);
        }

        loop {
            if remaining.len() <= 0 {
                return docs;
            }
            let next = remaining.pop_front().unwrap();
            if next.block_type == NodeType::DOCUMENT {
                docs.push(next);
                continue;
            }

            for block in (next.blocks).iter_mut() {
                remaining.push_back(block)
            }
        }
    }

    fn query(&mut self, query: &String) -> Vec<&Node> {
        let mut collected = vec![];
        let mut queue = VecDeque::new();
        for tree in self.trees.iter() {
            queue.push_back(tree);
        }
        while queue.len() > 0 {
            let next = queue.pop_front().unwrap();
            if next.type_as_string() == query.as_str() {
                collected.push(next)
            }
            for child in &next.blocks {
                queue.push_back(child);
            }
        }
        return collected;
    }
}

impl Node {
    async fn hydrate(&mut self, parser: &String) -> Result<(), std::io::ErrorKind> {
        if self.block_type != NodeType::DOCUMENT {
            return Err(std::io::ErrorKind::InvalidInput);
        }

        let output = Command::new(parser.as_str())
            .arg(self.data.get("path").unwrap())
            .output()
            .await
            .expect("");

        let parsed_json = String::from_utf8(output.stdout).expect("");
        let parsed_obj: Node = serde_json::from_str(parsed_json.as_str()).unwrap();

        self.raw = parsed_obj.raw;
        self.blocks = parsed_obj.blocks;

        return Ok(());
    }

    fn type_as_string(&self) -> &str {
        match self.block_type {
            NodeType::DIRECTORY => "directory",
            NodeType::HEADER => "header",
            NodeType::DOCUMENT => "document",
            NodeType::PARAGRAPH => "paragraph",
        }
    }
}

// current bugs:
// - paths involving . cause a crash

// current feature improvements:
// - zenkat should act as a server
//   - supporting queries and outputs
// - should be able to load multiple trees

async fn hydrate_docs(docs: Vec<&mut Node>, processes: usize, parser: &String) {
    let mut children: VecDeque<_> = VecDeque::new();
    let mut pending_doc: VecDeque<&mut Node> = VecDeque::new();
    let mut remaining_docs: Vec<&mut Node> = vec![];

    for doc in docs {
        remaining_docs.push(doc);
    }

    loop {
        if remaining_docs.len() == 0 && children.len() == 0 {
            break;
        }
        loop {
            if children.len() == processes || remaining_docs.len() == 0 {
                break;
            }
            let next_doc = remaining_docs.pop().expect("");

            let parser_clone = parser.clone();
            let next_doc_path = next_doc.data.get("path").unwrap().clone();

            pending_doc.push_back(next_doc);

            let child = tokio::spawn(Command::new(parser_clone).arg(next_doc_path).output());

            children.push_back(child);
        }
        let output = children.pop_front().unwrap().await.unwrap().unwrap();

        let parsed_json = String::from_utf8(output.stdout).expect("");
        let parsed_obj: Node = serde_json::from_str(parsed_json.as_str()).unwrap();

        let finished_doc = pending_doc.pop_front().unwrap();

        finished_doc.blocks = parsed_obj.blocks;
        finished_doc.raw = parsed_obj.raw;
    }
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
        hydrate_docs(docs, processes.into(), &parser).await;
    }
    if args.query.is_some() {
        let collected = store.query(&args.query.unwrap());
        let as_json = serde_json::to_string_pretty(&collected).unwrap();
        println!("{}", as_json);
    }
    if args.trees {
        for tree in store.trees {
            println!("{}", tree.data.get("path").unwrap());
        }
    }
}
