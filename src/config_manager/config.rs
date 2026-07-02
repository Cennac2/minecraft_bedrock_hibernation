use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub bedrock_server_port: u16,
    pub protocol_version: i16,
    pub version: String,
    pub hibernating_motd: String,
    pub bedrock_file_path: String,
    pub stop_empty_server_after_seconds: u32
}
fn create_default_config_file(path: &str) {
    let default_config = Config {
        hibernating_motd: String::from("Server is Hibernating"),
        port: 19132,
        protocol_version: -1,
        version: String::from("1.26.30"),
        bedrock_file_path: String::from("./bedrock_server"),
        bedrock_server_port: 19134,
        stop_empty_server_after_seconds: 60
    };

    let json_str = match serde_json::to_string_pretty(&default_config) {
        Ok(str) => str,
        Err(e) => {
            eprintln!("Failed to serialize default config: {}", e);
            return;
        }
    };

    if let Err(e) = fs::write(path, json_str) {
        eprintln!("Failed to create config file: {}", e);
    }
}

pub fn get_config() -> Config {
    let path = "mbh_config.json";

    if !Path::new(path).exists() {
        create_default_config_file(path);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read mbh_config.json: {}", e);
            return Config {
                hibernating_motd: String::from("Server is Hibernating"),
                port: 19132,
                protocol_version: -1,
                version: String::from("1.26.30"),
                bedrock_file_path: String::from("./bedrock_server"),
                bedrock_server_port: 19134,
                stop_empty_server_after_seconds: 60
            };
        }
    };

    match serde_json::from_str::<Config>(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse mbh_config.json: {}", e);
            Config {
                hibernating_motd: String::from("Server is Hibernating"),
                port: 19132,
                protocol_version: -1,
                version: String::from("1.26.30"),
                bedrock_file_path: String::from("./bedrock_server"),
                bedrock_server_port: 19134,
                stop_empty_server_after_seconds: 60
            }
        }
    }
}