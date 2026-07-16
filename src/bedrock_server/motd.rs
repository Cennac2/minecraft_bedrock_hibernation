use std::{
    net::{Ipv4Addr, SocketAddr::V4, SocketAddrV4},
    time::Duration,
};

use rust_raknet::RaknetSocket;
use tokio::time::timeout;

use crate::CONFIG;

#[allow(dead_code)] // will get rid of this later
#[derive(Debug)]
pub struct Motd {
    pub game_type: String,
    pub server_name: String,
    pub protocol_version: i16,
    pub minecraft_version: String,
    pub player_count: u32,
    pub max_player_count: u32,
    pub server_id: String,
    pub world_name: String,
    pub gamemode: String,
    pub numeric_gamemode: u8,
    pub port_v4: u16,
    pub port_v6: u16,
    // the rest dont matter
}

pub async fn get_server_motd_string() -> Option<String> {
    let config = &CONFIG;
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

pub async fn get_server_motd() -> Option<Motd> {
    let motd_string = get_server_motd_string().await;

    let mut result_motd = None;

    if let Some(motd) = motd_string {
        let parts: Vec<&str> = motd.split(';').collect();
        if parts.len() < 12 {
            return None;
        }

        result_motd = Some(Motd {
            game_type: parts[0].to_string(),
            server_name: parts[1].to_string(),
            protocol_version: parts[2].parse().ok()?,
            minecraft_version: parts[3].to_string(),
            player_count: parts[4].parse().ok()?,
            max_player_count: parts[5].parse().ok()?,
            server_id: parts[6].to_string(),
            world_name: parts[7].to_string(),
            gamemode: parts[8].to_string(),
            numeric_gamemode: parts[9].parse().ok()?,
            port_v4: parts[10].parse().ok()?,
            port_v6: parts[11].parse().ok()?,
        });
    }

    result_motd
}

pub fn get_motd_from_config() -> Motd {
    let config = &CONFIG;

    Motd {
        game_type: String::from("MCPE"),
        server_name: config.hibernating_motd.clone(),
        protocol_version: config.protocol_version,
        minecraft_version: config.version.clone(),
        player_count: 0,
        max_player_count: 1,
        server_id: String::new(),
        world_name: String::from("Bedrock level"),
        gamemode: String::from("Creative"),
        numeric_gamemode: 1, // I think creative is 1?
        port_v4: config.port,
        port_v6: 19133,
    }
}
