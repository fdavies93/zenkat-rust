use clap::Parser;
use std::io::{self, Write};

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn main() {
    let args = Args::parse();

    let output = format!("{{ \"path\": \"{}\" }}", &args.path);
    io::stdout().write_all(output.as_bytes()).unwrap();
}
