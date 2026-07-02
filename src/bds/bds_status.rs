use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rust_raknet::RaknetSocket;
use tokio::time::timeout;

pub async fn is_bedrock_server_online(addr: Ipv4Addr, port: u16, max_attempts: u32) -> bool {
    for _ in 1..=max_attempts {
        if ping_server(addr, port).await {
            return true;
        }
        if max_attempts > 1 { tokio::time::sleep(Duration::from_secs(1)).await; }
    }
    false
}

async fn ping_server(addr: Ipv4Addr, port: u16) -> bool {
    let sock_addr = SocketAddr::new(IpAddr::V4(addr), port);

    let result = timeout(
        Duration::from_secs(2),
        RaknetSocket::ping(&sock_addr),
    ).await;

    match result {
        Ok(Ok((latency, _))) => {
            latency >= 0
        }
        Ok(Err(_)) => false,
        Err(_) => {
            false
        }
    }
}