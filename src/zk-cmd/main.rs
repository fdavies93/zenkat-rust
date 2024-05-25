use clap::{Parser, Subcommand};
use reqwest;
use tokio;

#[path = "../common.rs"]
mod common;

use common::zk_request::{ZkRequest, ZkRequestType};

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,

    #[arg(long, default_value = "http")]
    protocol: String,

    #[arg(long, default_value = "9001")]
    port: String,

    #[arg(long, default_value = "localhost")]
    host: String,
}

#[derive(Debug, Subcommand)]
enum Command {
    LoadZk { path: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let server_uri = vec![
        args.protocol,
        "://".into(),
        args.host,
        ":".into(),
        args.port,
    ]
    .join("");

    match &args.cmd {
        Command::LoadZk { path } => {
            let mut request = ZkRequest::new(ZkRequestType::ZkLoad);
            request.data.insert("load_zk".to_string(), path.clone());
            let res = client.post(server_uri).json(&request).send().await;

            match res {
                Ok(res) => {
                    println!("{:?}", res)
                }
                Err(e) => {
                    println!("{:?}", e)
                }
            }
        }
    }
}
