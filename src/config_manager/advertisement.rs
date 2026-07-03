use crate::{bds::protocol::get_protocol_version, config_manager::config::Config};

#[allow(dead_code)]
pub enum Gamemode {
    Survival = 0,
    Creative,
    Adventure,
    Spectator
}

impl Gamemode {
    pub fn as_str(&self) -> &str {
        match self {
            Gamemode::Survival => "Survival",
            Gamemode::Creative => "Creative",
            Gamemode::Adventure => "Adventure",
            Gamemode::Spectator => "Spectator",
        }
    }
}

pub struct Motd {
    /// The name of the server
    pub name: String,
    /// The protocol version
    pub protocol: u16,
    /// The version of the server
    pub version: String,
    /// The maximum number of players
    pub max_players: u32,
    /// The gamemode of the server
    pub gamemode: Gamemode,
    /// The server's IPv4 port
    pub port: u16,
}

pub fn get_advertisement(config: Config) -> Motd {
    let protocol_version = if config.protocol_version > 0 { config.protocol_version as u16 } else { get_protocol_version(&config.version).unwrap_or(0) };
    return Motd {
        gamemode: Gamemode::Creative,
        name: config.hibernating_motd,
        max_players: 2,
        port: config.port,
        protocol: protocol_version,
        version: config.version,
    };
}