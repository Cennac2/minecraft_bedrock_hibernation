use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    bedrock_server::bedrock_server_child::SharedBedrockServer, config::config::get_config,
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

#[tokio::main]
async fn main() {
    let config = get_config();
    let shared_bedrock_server: SharedBedrockServer = Arc::new(Mutex::new(None));

    println!("[MBH] Starting up MBH");

    start_proxy(config, shared_bedrock_server).await;

    println!("[MBH] MBH Stopped");
}
