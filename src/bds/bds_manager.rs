use std::net::Ipv4Addr;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::bds::bds_status::is_bedrock_server_online;
use crate::bds::console_io::{handle_bds_error, handle_bds_output};
use crate::config_manager::config::Config;
use crate::proxy::proxy::get_startup_message;

pub struct BedrockServer {
    pub child: Child,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
    hibernate_handle: Option<JoinHandle<()>>
}

pub type SharedChild = Arc<Mutex<Option<Arc<Mutex<BedrockServer>>>>>;

pub async fn start_bedrock_server(config: &Config, counter: Arc<Mutex<u32>>) -> Arc<Mutex<BedrockServer>> {
    update_bedrock_server_port(config.bedrock_server_port);
    println!("Running bedrock server file.");
    let mut child = match Command::new(&config.bedrock_file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("{:?}", e);
                panic!();
            }
        };

    let stdout_handle = child.stdout.take().map(|stdout| {
        tokio::spawn(async move {
            handle_bds_output(stdout).await;
        })
    });

    let stderr_handle = child.stderr.take().map(|stderr| {
        tokio::spawn(async move {
            handle_bds_error(stderr).await;
        })
    });

    let server = Arc::new(Mutex::new(BedrockServer {
        child,
        stdout_handle,
        stderr_handle,
        hibernate_handle: None
    }));

    let hibernate_handle = tokio::spawn(start_should_hibernate_check_loop(
        Arc::clone(&server),
        config.stop_empty_server_after_seconds,
        counter,
    ));

    {
        let mut guard = server.lock().await;
        guard.hibernate_handle = Some(hibernate_handle);
    }

    server
}

pub async fn start_should_hibernate_check_loop(
    server: Arc<Mutex<BedrockServer>>,
    duration: u32,
    counter: Arc<Mutex<u32>>,
) {
    loop {
        let mut clients_amount = *counter.lock().await;

        if clients_amount == 0 {
            tokio::time::sleep(Duration::from_secs(duration as u64)).await;

            clients_amount = *counter.lock().await;

            if clients_amount == 0 {
                println!("[MBH] No players connected for {} seconds, stopping server..", duration);
                
                let mut guard = server.lock().await;
                guard.hibernate_handle.take();
                stop_bedrock_server(&mut guard).await;
                break;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

fn update_bedrock_server_port(port: u16) {
    println!("Updating server port to {port}");

    let path = "server.properties";
    let key = "server-port";
    let new_value = port.to_string();
    
    let contents = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut found = false;

    for line in lines.iter_mut() {
        let trimmed = line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }

        if let Some(sep_idx) = trimmed.find(['=', ':']) {
            let existing_key = trimmed[..sep_idx].trim();
            if existing_key == key {
                *line = format!("{key}={new_value}");
                found = true;
                break;
            }
        }
    }

    if !found {
        lines.push(format!("{key}={new_value}"));
    }

    let output = lines.join("\n") + "\n";

    if let Err(e) = std::fs::write(path, output) {
        eprintln!("Failed to write properties file: {}", e);
    }
}

pub async fn get_main_child(mut server: Option<Arc<Mutex<BedrockServer>>>, config: &Config, counter: Arc<Mutex<u32>>) -> Arc<Mutex<BedrockServer>> {
    let server_online = is_bedrock_server_online(Ipv4Addr::LOCALHOST, config.bedrock_server_port, 1).await;

    let is_active = match &mut server {
        Some(s) => {
            let mut s = s.lock().await;
            matches!(s.child.try_wait(), Ok(None))
        },
        None => false,
    };

    if server_online && is_active {
        println!("[MBH] Bedrock Server already online!");
        server.unwrap()
    } else {
        println!("[MBH] Starting up Bedrock Server!");
        start_bedrock_server(config, counter).await
    }
}

pub async fn stop_bedrock_server(server: &mut BedrockServer) {
    if let Some(handle) = server.hibernate_handle.take() {
        handle.abort();
    }

    if let Some(stdin) = server.child.stdin.as_mut() {
        if let Err(e) = stdin.write_all(b"stop\n").await {
            eprintln!("[MBH] Failed to write stop command: {:?}", e);
        }
        let _ = stdin.flush().await;
    } else {
        eprintln!("[MBH] No stdin handle for Bedrock Server, killing instead.");
        let _ = server.child.start_kill();
    }

    println!("[MBH] Waiting for Bedrock Server to exit...");
    match server.child.wait().await {
        Ok(_) => {},
        Err(e) => eprintln!("[MBH] Error while waiting for Bedrock Server to exit: {:?}", e),
    }

    if let Some(handle) = server.stdout_handle.take() {
        let _ = handle.await;
    }
    if let Some(handle) = server.stderr_handle.take() {
        let _ = handle.await;
    }
}