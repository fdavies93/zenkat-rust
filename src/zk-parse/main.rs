use clap::Parser;
use std::io::{self, Write};
use std::{thread, time};

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn main() {
    let duration = time::Duration::from_millis(250);
    thread::sleep(duration);
    let args = Args::parse();
    let output = format!("{{ \"path\": \"{}\" }}", &args.path);
    io::stdout().write_all(output.as_bytes()).unwrap();
}
