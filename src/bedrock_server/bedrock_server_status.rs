use std::{
    net::{Ipv4Addr, SocketAddr::V4, SocketAddrV4},
    time::Duration,
};

use rust_raknet::RaknetSocket;
use tokio::time::timeout;

use crate::{bedrock_server::bedrock_server_child::SharedBedrockServer, config::config::Config};

pub async fn is_bedrock_server_alive(bedrock_server: SharedBedrockServer) -> bool {
    let mut guard = bedrock_server.lock().await;

    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => false,
            _ => true,
        }
    } else {
        false
    }
}

pub async fn is_bedrock_server_online(config: Config) -> bool {
    let addr = &V4(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        config.bedrock_server_port,
    ));

    for _ in 1..=5 {
        match timeout(Duration::from_secs(1), RaknetSocket::ping(addr)).await {
            Ok(Ok((latency, _))) => {
                if latency >= 0 {
                    return true;
                }
            }
            Ok(Err(_)) => {}
            Err(_) => {}
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    false
}

pub async fn get_server_motd(config: Config) -> Option<String> {
    let addr = &V4(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        config.bedrock_server_port,
    ));

    for _ in 1..=5 {
        match timeout(Duration::from_secs(2), RaknetSocket::ping(addr)).await {
            Ok(Ok((latency, motd))) => {
                if latency >= 0 && !motd.is_empty() {
                    return Some(motd);
                }
            }
            Ok(Err(_)) => {}
            Err(_) => {}
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    None
}
