use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn walk(path: &Path) {
    for entry in path.read_dir().expect("Couldn't read the directory.") {}
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.path);

    if !path.try_exists().is_ok_and(|x| x) {:qa
        println!("Couldn't access path at {}", args.path);
        return;
    }

    let mut vec: Vec<&Path> = Vec::new();

    if path.is_file() && path.extension().unwrap() == "md" {
        vec.push(path)
    } else if path.is_dir() {
        // crawl the directory structure
    }

    println!("{}", args.path);
}
