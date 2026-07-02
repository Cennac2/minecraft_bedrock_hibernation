use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use rust_raknet::{RaknetListener, RaknetSocket};

use crate::bds::bds_manager::{SharedChild, get_main_child, stop_bedrock_server};
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

pub async fn start_proxy() {
    let config = get_config();

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    
    let mut server = match RaknetListener::bind(&sock_addr).await {
        Ok(srv) => srv,
        Err(e) => {
            println!("Failed to start server!");
            panic!("Error: {:?}", e);
        }
    };

    let advert = get_advertisement(config.clone());

    server.listen().await;

    server.set_motd(&advert.name, advert.max_players, &advert.protocol.to_string(), &advert.version, advert.gamemode.as_str(), advert.port).await;

    println!("{}", get_startup_message());

    let child_state: SharedChild = Arc::new(Mutex::new(None));

    let shutdown_child_state = child_state.clone();
    tokio::spawn(async move {
        handle_exit(shutdown_child_state).await;
    });

    let clients_amount: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    let console_child_state = child_state.clone();
    let console_config = config.clone();
    let console_clients_amount = Arc::clone(&clients_amount);
    tokio::spawn(async move {
        handle_user_input(console_child_state, console_config, console_clients_amount).await;
    });

    tokio::task::spawn(handle_status());
    proxy_handle_connections(server, config, child_state, Arc::clone(&clients_amount)).await;
}

async fn handle_exit(shutdown_child: SharedChild) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("[MBH] Failed to listen for Ctrl+C: {:?}", e);
        return;
    }

    println!("\n[MBH] Ctrl+C received, shutting down...");

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

async fn handle_status() {
    let mut was_online = false;

    loop {
        let online = is_bedrock_server_online(Ipv4Addr::LOCALHOST, get_config().bedrock_server_port, 5).await;

        if online && !was_online {
            println!("[MBH] Bedrock server started successfully.");
        }

        if !online && was_online {
            println!("[MBH] Bedrock server stopped.");
        }

        was_online = online;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

pub async fn proxy_handle_connections(mut server: RaknetListener, config: Config, child_state: SharedChild, clients_amount: Arc<Mutex<u32>>) {
    let mut raknet_version: Option<u8> = None;
    loop {
        let conn = server.accept().await;

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
                        
                        // TODO: Make Server kick player and display a message.
                        c.close().await.unwrap();
                    } else {
                        let bds_addr = &SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.bedrock_server_port);

                        if raknet_version == None {
                            for version in 0..=20 {
                                match RaknetSocket::connect_with_version(&bds_addr, version).await {
                                    Ok(_) => {
                                        println!("[MBH] Worked with version {}", version);
                                        raknet_version = Some(version);
                                        break;
                                    }
                                    Err(e) => print!("[MBH] {} -> {:?}, ", version, e),
                                }
                            }
                        }

                        let bds_client = match RaknetSocket::connect_with_version(
                            &SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.bedrock_server_port), raknet_version.unwrap()
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