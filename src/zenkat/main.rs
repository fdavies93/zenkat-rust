use clap::Parser;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Parser, Debug)]
struct Args {
    path: String,
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
    println!("{}", vec.len());

    return vec;
}

async fn parse_at_paths(paths: Vec<PathBuf>, processes: usize, parser: &str) -> Vec<String> {
    let mut parsed: Vec<String> = vec![];
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
            let child = Command::new(parser).arg(next_path).output();
            children.push_back(child);
        }
        let output = children.pop_front().expect("").await.expect("");
        let string = String::from_utf8(output.stdout).expect("");
        parsed.push(string);
    }
    return parsed;
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let path = Path::new(&args.path);

    let parser = "target/debug/zk-parse";

    if !path.try_exists().is_ok_and(|x| x) {
        println!("Couldn't access path at {}", args.path);
        return;
    }

    let mut vec: Vec<PathBuf> = Vec::new();

    let pathbuf = path.canonicalize().expect("");
    let path = pathbuf.as_path();
    println!("{:?}", path);
    if path.is_file() && path.extension().unwrap() == "md" {
        vec.push(path.to_path_buf())
    } else if path.is_dir() {
        // crawl the directory structure
        let paths = walk(path, false);
        vec.extend(paths.into_iter());
    }

    println!(
        "Found {} markdown files in {}, parsing.",
        vec.len(),
        &args.path
    );

    let parsed = parse_at_paths(vec, 4, parser).await;
    println!("{:?}", parsed);
}
