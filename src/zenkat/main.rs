use clap::Parser;
use serde_json::from_str;
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
    paths: Vec<String>,

    #[arg(short, long, default_value = "0")]
    processes: usize,

    #[arg(long, default_value = "")]
    parser: String,
}

struct TreeStore {
    trees: Vec<Node>,
}

fn walk(path: &Path, follow_symlinks: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut vec: Vec<PathBuf> = Vec::new();

    // bfs for directories
    let mut paths = VecDeque::from(vec![path.to_path_buf()]);
    let err = std::io::ErrorKind::InvalidInput;
    loop {
        if paths.len() == 0 {
            break;
        }
        let current_path = paths.pop_front().ok_or(err)?;

        for entry in current_path.read_dir()? {
            if let Ok(entry) = entry {
                let pathbuf = entry.path();
                let entry_path = pathbuf.as_path();
                if entry_path.is_file() && entry_path.extension().ok_or(err)? == "md" {
                    vec.push(pathbuf);
                } else if entry_path.is_dir() {
                    paths.push_back(entry_path.to_path_buf());
                // for unknown reason seems to treat symlinks like normal directories :o
                } else if entry_path.is_symlink() && follow_symlinks {
                    let dest = entry_path.read_link()?;
                    let symlink_path = dest.as_path();
                    if symlink_path.is_dir() {
                        paths.push_back(dest);
                    } else if symlink_path.is_file() && symlink_path.extension().ok_or(err)? == "md"
                    {
                        vec.push(dest);
                    }
                }
            }
        }
    }

    return Ok(vec);
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

async fn load_tree_store(paths: Vec<String>, processes: NonZeroUsize, parser: String) -> TreeStore {
    let mut vec: Vec<PathBuf> = Vec::new();
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

        let pathbuf = path.canonicalize().expect("");
        let path = pathbuf.as_path();
        if path.is_file() && path.extension().unwrap() == "md" {
            vec.push(path.to_path_buf())
        } else if path.is_dir() {
            // crawl the directory structure
            let paths = walk(path, false).unwrap();
            vec.extend(paths.into_iter());
        }

        println!(
            "Found {} markdown files in {}, parsing with {} processes.",
            vec.len(),
            path.to_str().unwrap(),
            processes
        );
    }

    // after the paths are discovered and parsed, we need to make sure they're put back into the Node tree in the correct places

    let parsed = parse_at_paths(vec, processes.into(), parser).await;
    store.trees = parsed;

    return store;
}

// current bugs:
// - paths involving . cause a crash

// current feature improvements:
// - zenkat should act as a server
//   - supporting queries and outputs
// - should be able to load multiple trees

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

    let store = load_tree_store((&args.paths).clone(), processes, parser).await;
    // println!("{:?}", parsed);
}
