use std::{path::Path, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    bedrock_server::bedrock_server_child::SharedBedrockServer,
    config::config::{Config, get_config},
    proxy::proxy::start_proxy,
};

mod bedrock_server;
mod config;
mod proxy;

const MBH_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_startup_message() -> String {
    format!(
        r"
 __  __ ____  _   _
|  \/  | __ )| | | |
| |\/| |  _ \| |_| |
| |  | | |_) |  _  |
|_|  |_|____/|_| |_| {MBH_VERSION}

Minecraft Bedrock (server) Hibernation
---------------------------------------
Server is hibernating, join to start it up.
"
    )
}

pub fn do_startup_checks(config: Config) {
    if !Path::new(&config.bedrock_file_path).exists() {
        eprintln!("[MBH] File '{}' not found.", config.bedrock_file_path);
        std::process::exit(1);
    }

    if config.bedrock_server_port == config.port {
        eprintln!("[MBH] Error: Bedrock server port and proxy port should not be equal!");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let config = get_config();
    do_startup_checks(config.clone());
    let shared_bedrock_server: SharedBedrockServer = Arc::new(Mutex::new(None));

    println!("[MBH] Starting up MBH");

    start_proxy(config, shared_bedrock_server).await;

    println!("[MBH] MBH Stopped");
}
