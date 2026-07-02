mod proxy;
mod config_manager;
mod bds;

#[tokio::main]
async fn main() {
    println!("Starting up Minecraft Bedrock Hibernation");
    
    proxy::proxy::start_hibernating_proxy().await;
}