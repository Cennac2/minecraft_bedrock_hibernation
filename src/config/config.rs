use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "mbh_config.json";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_bedrock_server_port")]
    pub bedrock_server_port: u16,
    #[serde(default = "default_protocol_version")]
    pub protocol_version: i16,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_hibernating_motd")]
    pub hibernating_motd: String,
    #[serde(default = "default_bedrock_file_path")]
    pub bedrock_file_path: String,
    #[serde(default = "default_stop_empty_server_after_seconds")]
    pub stop_empty_server_after_seconds: u32,
}

fn default_port() -> u16 {
    19132
}
fn default_bedrock_server_port() -> u16 {
    19134
}
fn default_protocol_version() -> i16 {
    -1
}
fn default_version() -> String {
    String::from("auto")
}
fn default_hibernating_motd() -> String {
    String::from("Server is Hibernating")
}
fn default_bedrock_file_path() -> String {
    #[cfg(target_os = "windows")]
    {
        String::from(r".\bedrock_server.exe")
    }

    #[cfg(not(target_os = "windows"))]
    {
        String::from("./bedrock_server")
    }
}
fn default_stop_empty_server_after_seconds() -> u32 {
    60
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: default_port(),
            bedrock_server_port: default_bedrock_server_port(),
            protocol_version: default_protocol_version(),
            version: default_version(),
            hibernating_motd: default_hibernating_motd(),
            bedrock_file_path: default_bedrock_file_path(),
            stop_empty_server_after_seconds: default_stop_empty_server_after_seconds(),
        }
    }
}

pub fn get_config() -> Config {
    let path = Path::new(CONFIG_PATH);

    if !path.exists() {
        let default_config = Config::default();

        match serde_json::to_string_pretty(&default_config) {
            Ok(serialized) => {
                if let Err(e) = fs::write(path, serialized) {
                    eprintln!("[MBH] Failed to create config file: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[MBH] Failed to serialize default config: {}", e);
            }
        }

        return default_config;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[MBH] Failed to read {}: {}", CONFIG_PATH, e);
            std::process::exit(1);
        }
    };

    let config: Config = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("[MBH] Failed to parse {}: {}", CONFIG_PATH, e);
            std::process::exit(1);
        }
    };

    match serde_json::to_string_pretty(&config) {
        Ok(serialized) => {
            if let Err(e) = fs::write(path, serialized) {
                eprintln!("[MBH] Failed to re-sync config file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[MBH] Failed to serialize config for re-sync: {}", e);
        }
    }

    config
}
