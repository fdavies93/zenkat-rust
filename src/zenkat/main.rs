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
    path: String,

    #[arg(short, long, default_value = "0")]
    processes: usize,

    #[arg(long, default_value = "")]
    parser: String,
}

fn walk(path: &Path, follow_symlinks: bool) -> Vec<PathBuf> {
    let mut vec: Vec<PathBuf> = Vec::new();

    // bfs for directories
    let mut paths = VecDeque::from(vec![path.to_path_buf()]);
    loop {
        if paths.len() == 0 {
            break;
        }
        let current_path = paths.pop_front().expect("");

        // might want to check with pathbufs whether this causes a memory leak due to copies

        for entry in current_path.read_dir().expect("read_dir failed") {
            if let Ok(entry) = entry {
                let pathbuf = entry.path();
                let entry_path = pathbuf.as_path();
                if entry_path.is_file() && entry_path.extension().expect("") == "md" {
                    vec.push(pathbuf);
                } else if entry_path.is_dir() {
                    paths.push_back(entry_path.to_path_buf());
                // for unknown reason seems to treat symlinks like normal directories :o
                } else if entry_path.is_symlink() && follow_symlinks {
                    let dest = entry_path.read_link().expect("Symlink failure");
                    let symlink_path = dest.as_path();
                    if symlink_path.is_dir() {
                        paths.push_back(dest);
                    } else if symlink_path.is_file() && symlink_path.extension().expect("") == "md"
                    {
                        vec.push(dest);
                    }
                }
                // no symlink support for now
            }
        }
    }

    return vec;
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
        let parsed_string = serde_json::from_str(string.as_str()).expect("");
        parsed.push(parsed_string);
    }
    return parsed;
}

#[tokio::main]
async fn main() {
    let mut processes = thread::available_parallelism().expect("");
    let mut parser = String::from("target/debug/md-parse");

    let args = Args::parse();
    let path = Path::new(&args.path);

    if args.processes > 0 {
        processes = NonZeroUsize::new(args.processes).expect("");
    }

    if args.parser.len() > 0 {
        parser = args.parser;
    }

    if !path.try_exists().is_ok_and(|x| x) {
        println!("Couldn't access path at {}", args.path);
        return;
    }

    let mut vec: Vec<PathBuf> = Vec::new();

    let pathbuf = path.canonicalize().expect("");
    let path = pathbuf.as_path();
    if path.is_file() && path.extension().unwrap() == "md" {
        vec.push(path.to_path_buf())
    } else if path.is_dir() {
        // crawl the directory structure
        let paths = walk(path, false);
        vec.extend(paths.into_iter());
    }

    println!(
        "Found {} markdown files in {}, parsing with {} processes.",
        vec.len(),
        &args.path,
        processes
    );

    let parsed = parse_at_paths(vec, processes.into(), parser).await;
    println!("{:?}", parsed);
}
