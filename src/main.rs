use crate::{config::config::get_config, proxy::proxy::start_proxy};

mod proxy;
mod config;
mod protocol_version;

const MBH_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_startup_message() -> String {
    format!(r"
 __  __ ____  _   _
|  \/  | __ )| | | |
| |\/| |  _ \| |_| |
| |  | | |_) |  _  |
|_|  |_|____/|_| |_| {MBH_VERSION}

Minecraft Bedrock (server) Hibernation
---------------------------------------
Server is hibernating, join to start it up.
")
}

#[tokio::main]
async fn main() {
    let config = get_config();

    println!("Starting up MBH");

    start_proxy(config).await;

    println!("MBH Stopped");
}