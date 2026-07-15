use std::{
    io::stdout,
    net::{Ipv4Addr, SocketAddr::V4, SocketAddrV4},
    sync::{Arc, atomic::AtomicU32},
    time::Duration,
};

use crate::{
    bedrock_server::{
        bedrock_server_child::{
            SharedBedrockServer, start_bedrock_server, start_server_then_get_motd,
        },
        bedrock_server_io::handle_user_input,
        bedrock_server_status::{get_server_motd, is_bedrock_server_alive},
    },
    config::config::Config,
    get_startup_message,
    proxy::proxy_connector::start_proxy_connection,
};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use rust_raknet::{RaknetListener, RaknetSocket};
use tokio::sync::RwLock;

pub async fn start_proxy(config: Config, shared_bedrock_server: SharedBedrockServer) {
    let port = config.port;

    let sockaddr = &V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));
    let mut proxy = match RaknetListener::bind(sockaddr).await {
        Ok(p) => p,
        Err(e) => {
            println!("[MBH] Failed to bind on port {port}: {:?}", e);
            std::process::exit(1);
        }
    };

    let motd_parts: Option<Vec<String>> =
        if config.version == "auto" || config.protocol_version <= 0 {
            let motd = start_server_then_get_motd(config.clone())
                .await
                .unwrap_or_else(|| {
                    eprintln!("[MBH] Failed to get minecraft version automatically!");
                    std::process::exit(1);
                });

            Some(motd.split(';').map(String::from).collect())
        } else {
            None
        };

    let minecraft_version = if config.version == "auto" {
        motd_parts
            .as_ref()
            .and_then(|parts| parts.get(3))
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("[MBH] Failed to parse minecraft version from MOTD!");
                std::process::exit(1);
            })
    } else {
        config.version.clone()
    };

    let protocol_version = if config.protocol_version > 0 {
        config.protocol_version as u16
    } else {
        motd_parts
            .as_ref()
            .and_then(|parts| parts.get(2))
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| {
                eprintln!("[MBH] Failed to parse protocol version from MOTD!");
                std::process::exit(1);
            })
    };

    println!("{}", get_startup_message());

    proxy.listen().await;

    proxy
        .set_motd(
            &config.hibernating_motd,
            2,
            &protocol_version.to_string(),
            &minecraft_version,
            "Creative",
            port,
        )
        .await;

    let default_motd = proxy.get_motd().await;

    tokio::spawn(handle_user_input(
        shared_bedrock_server.clone(),
        config.clone(),
    ));

    tokio::spawn(send_startup_message_if_offline(
        shared_bedrock_server.clone(),
    ));

    let motd_handle = proxy.motd_handle();

    tokio::spawn(update_server_motd(
        motd_handle,
        default_motd,
        config.clone(),
    ));

    proxy_loop(proxy, shared_bedrock_server, config).await;
}

async fn update_server_motd(
    motd_handle: Arc<RwLock<String>>,
    hibernating_motd: String,
    config: Config,
) {
    loop {
        let server_motd = get_server_motd(config.clone()).await;

        if let Some(motd) = server_motd {
            *motd_handle.write().await = motd;
        } else {
            *motd_handle.write().await = hibernating_motd.clone();
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn send_startup_message_if_offline(server: SharedBedrockServer) {
    let mut was_active = false;

    loop {
        let active = is_bedrock_server_alive(server.clone()).await;
        if !active && was_active {
            execute!(stdout(), Clear(ClearType::All)).unwrap();
            println!("{}", get_startup_message());
        }

        was_active = active;

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub static PLAYERS_COUNTER: AtomicU32 = AtomicU32::new(0);

const RAKNET_VERSION: u8 = 11; // I don't think this ever changed so I'm keeping it as 11

pub async fn proxy_loop(mut proxy: RaknetListener, server: SharedBedrockServer, config: Config) {
    let bds_addr = &V4(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        config.bedrock_server_port,
    ));

    loop {
        let connection = match proxy.accept().await {
            Ok(c) => c,
            Err(e) => {
                println!("[MBH] Failed to accept request: {:?}", e);
                return;
            }
        };

        println!(
            "[MBH] Player Connected from {}",
            connection
                .peer_addr()
                .unwrap_or(V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1)))
        );

        if let Ok(packet) = connection.recv().await {
            let active = is_bedrock_server_alive(server.clone()).await;

            if !active {
                start_bedrock_server(server.clone(), config.clone()).await;
            }

            let server_client =
                match RaknetSocket::connect_with_version(bds_addr, RAKNET_VERSION).await {
                    Ok(s) => s,
                    Err(e) => {
                        println!("[MBH] Failed to connect to bedrock_server: {:?}", e);
                        continue;
                    }
                };

            server_client
                .send(&packet, rust_raknet::Reliability::ReliableOrdered)
                .await
                .unwrap();

            tokio::spawn(start_proxy_connection(connection, server_client));
        }
    }
}
