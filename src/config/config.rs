use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

fn default_port() -> u16 { 19132 }
fn default_bedrock_server_port() -> u16 { 19134 }
fn default_protocol_version() -> i16 { -1 }
fn default_version() -> String { String::from("1.26.30") }
fn default_hibernating_motd() -> String { String::from("Server is Hibernating") }
fn default_bedrock_file_path() -> String { String::from("./bedrock_server") }
fn default_stop_empty_server_after_seconds() -> u32 { 60 }

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

fn write_config(path: &str, config: &Config) {
    match serde_json::to_string_pretty(config) {
        Ok(json_str) => {
            if let Err(e) = fs::write(path, json_str) {
                eprintln!("Failed to write config file: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to serialize config: {}", e),
    }
}

pub fn get_config() -> Config {
    let path = "mbh_config.json";

    if !Path::new(path).exists() {
        let default_config = Config::default();
        write_config(path, &default_config);
        return default_config;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read mbh_config.json: {}", e);
            return Config::default();
        }
    };

    let raw: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse mbh_config.json: {}", e);
            let default_config = Config::default();
            write_config(path, &default_config);
            return default_config;
        }
    };

    let config: Config = match serde_json::from_value(raw.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse mbh_config.json: {}", e);
            let default_config = Config::default();
            write_config(path, &default_config);
            return default_config;
        }
    };

    let resolved_value = serde_json::to_value(&config).unwrap_or(Value::Null);
    if let (Value::Object(raw_obj), Value::Object(resolved_obj)) = (&raw, &resolved_value) {
        let was_missing_fields = resolved_obj.keys().any(|k| !raw_obj.contains_key(k));
        if was_missing_fields {
            println!("[MBH] Config file was missing fields, updating with defaults..");
            write_config(path, &config);
        }
    }

    config
}