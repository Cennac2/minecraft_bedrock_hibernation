use std::io::stdout;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use rust_raknet::error::RaknetError;
use tokio::sync::{Mutex, RwLock};
use rust_raknet::{RaknetListener, RaknetSocket};
use std::path::Path;
use crossterm::{execute, terminal::{Clear, ClearType}};

use crate::bds::bds_manager::{SharedChild, get_main_child, stop_bedrock_server};
use crate::bds::bds_status::get_bedrock_server_motd;
use crate::bds::console_io::handle_user_input;
use crate::bds::proxy_connector::proxy_connection;
use crate::config_manager::config::Config;
use crate::{bds::bds_status::is_bedrock_server_online, config_manager::{advertisement::get_advertisement, config::get_config}};

pub fn get_startup_message() -> String {
    String::from(
r"
 __  __ ____  _   _
|  \/  | __ )| | | |
| |\/| |  _ \| |_| |
| |  | | |_) |  _  |
|_|  |_|____/|_| |_|

Minecraft Bedrock Hibernation
------------------------------
Server is hibernating, join to start it up.
")
}

pub fn do_startup_checks() {
    let config = get_config();

    if !Path::new(&config.bedrock_file_path).exists() {
        eprintln!("File '{}' not found.", config.bedrock_file_path);
        panic!();
    } 
}

pub async fn start_hibernating_proxy() {
    do_startup_checks();

    let config = get_config();

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    
    let shared_server: Arc<Mutex<RaknetListener>> = match RaknetListener::bind(&sock_addr).await {
        Ok(srv) => Arc::new(Mutex::new(srv)),
        Err(e) => {
            println!("Failed to start server!");
            panic!("Error: {:?}", e);
        }
    };

    let advert = get_advertisement(config.clone());
    let default_motd;

    {
        let mut server = shared_server.lock().await;

        server.listen().await;

        server.set_motd(&advert.name, advert.max_players, &advert.protocol.to_string(), &advert.version, advert.gamemode.as_str(), advert.port).await;

        default_motd = server.get_motd().await;
    }

    println!("{}", get_startup_message());

    let child_state: SharedChild = Arc::new(Mutex::new(None));

    let shutdown_child_state = child_state.clone();
    tokio::spawn(handle_exit(shutdown_child_state));

    let clients_amount: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    let console_child_state = child_state.clone();
    let console_config = config.clone();
    let console_clients_amount = Arc::clone(&clients_amount);
    tokio::spawn(handle_user_input(console_child_state, console_config, console_clients_amount));

    let motd_handle = {
        let server = shared_server.lock().await;
        server.motd_handle()
    };
    tokio::spawn(update_server_motd(motd_handle, config.clone(), default_motd));

    tokio::spawn(send_startup_message_if_offline(config.clone()));

    proxy_handle_connections(shared_server, config, child_state, Arc::clone(&clients_amount)).await;
}

async fn send_startup_message_if_offline(config: Config) {
    let mut was_online = false;

    loop {
        let online = is_bedrock_server_online(Ipv4Addr::LOCALHOST, config.bedrock_server_port, 1).await;
        if !online && was_online {
            execute!(stdout(), Clear(ClearType::All)).unwrap();
            println!("{}", get_startup_message());
        }

        was_online = online;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn update_server_motd(motd_handle: Arc<RwLock<String>>, config: Config, hibernating_motd: String) {

    loop {
        let server_motd = get_bedrock_server_motd(Ipv4Addr::LOCALHOST, config.bedrock_server_port).await;

        if server_motd.is_empty() {
            *motd_handle.write().await = hibernating_motd.clone();
        } else {
            *motd_handle.write().await = server_motd;
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn handle_exit(shutdown_child: SharedChild) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("[MBH] Failed to listen for Ctrl+C: {:?}", e);
        return;
    }

    let guard = shutdown_child.lock().await;
    if let Some(server) = guard.as_ref() {
        let mut server = server.lock().await;
        stop_bedrock_server(&mut *server).await;
    } else {
        println!("[MBH] No running Bedrock Server to stop.");
    }

    println!("[MBH] Shutdown complete.");
    std::process::exit(0);
}

pub async fn proxy_handle_connections(shared_server: Arc<Mutex<RaknetListener>>, config: Config, child_state: SharedChild, clients_amount: Arc<Mutex<u32>>) {
    let mut raknet_version: Option<u8> = None;
    loop {
        let conn: Result<RaknetSocket, RaknetError> = {
            let mut server = shared_server.lock().await;
            server.accept().await
        };

        match conn {
            Ok(c) => {
                println!("[MBH] Player Connected from {}", c.peer_addr().unwrap());
                if let Ok(packet) = c.recv().await {
                    println!("[MBH] Player Connected Successfully!");
                    let online = is_bedrock_server_online(Ipv4Addr::LOCALHOST, config.bedrock_server_port, 1).await;

                    if !online {
                        {
                            let mut guard = child_state.lock().await;
                            let current = guard.take();
                            *guard = Some(get_main_child(current, &config, Arc::clone(&clients_amount)).await);
                        }
                    } 
                    {
                        let bds_addr = &SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.bedrock_server_port);

                        if raknet_version == None {
                            for version in 0..=20 {
                                match RaknetSocket::connect_with_version(&bds_addr, version).await {
                                    Ok(_) => {
                                        println!("");
                                        println!("[MBH] Worked with version {}", version);
                                        raknet_version = Some(version);
                                        break;
                                    }
                                    Err(e) => print!("[MBH] {} -> {:?}, ", version, e),
                                }
                            }
                        }

                        let bds_client = match RaknetSocket::connect_with_version(
                            bds_addr, raknet_version.unwrap()
                        ).await {
                            Ok(s) => s,
                            Err(e) => {
                                println!("Failed to connect to bedrock_server: {:?}", e);
                                continue;
                            }
                        };

                        bds_client
                            .send(&packet, rust_raknet::Reliability::ReliableOrdered)
                            .await
                            .unwrap();

                        tokio::spawn(proxy_connection(c, bds_client, Arc::clone(&clients_amount)));
                    }
                }
            },
            Err(e) => println!("Connection failed: {:?}", e), 
        }
    }
}