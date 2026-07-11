use std::net::{Ipv4Addr, SocketAddr::V4, SocketAddrV4};

use rust_raknet::RaknetListener;

use crate::{config::config::Config, get_startup_message, protocol_version::get_protocol_version};

pub async fn start_proxy(config: Config) {
    let port = 19132;

    let sockaddr = V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));
    let mut proxy = match RaknetListener::bind(&sockaddr).await {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to bind on port {port}: {:?}", e);
            panic!();
        }
    };

    let protocol_version = if config.protocol_version > 0 {
        config.protocol_version as u16
    } else {
        get_protocol_version(&config.version).unwrap_or(0) 
    };

    proxy.set_motd(&config.hibernating_motd, 2, &protocol_version.to_string(), &config.version, "Creative", port).await;

    proxy.listen().await;

    println!("{}", get_startup_message());
}