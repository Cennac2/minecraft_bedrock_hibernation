use std::sync::Arc;

use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::{ChildStderr, ChildStdout}, sync::Mutex};

use crate::bds::bds_manager::{SharedChild, get_main_child, stop_bedrock_server};
use crate::config_manager::config::Config;

pub async fn handle_bds_output(stdout: ChildStdout) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        println!("BDS > {line}");
    }
}

pub async fn handle_bds_error(stderr: ChildStderr) {
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        eprintln!("BDS Error > {line}");
    }
}

pub async fn handle_user_input(child_state: SharedChild, config: Config, counter: Arc<Mutex<u32>>) {
    let reader = BufReader::new(tokio::io::stdin());
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.starts_with("mbh") {
            let command = line.trim_start_matches("mbh").trim();

            match command {
                "start" => {
                    let mut guard = child_state.lock().await;

                    let already_running = match guard.as_ref() {
                        Some(server) => {
                            let mut s = server.lock().await;
                            matches!(s.child.try_wait(), Ok(None))
                        }
                        None => false,
                    };

                    if already_running {
                        println!("[MBH] Bedrock Server is already running.");
                    } else {
                        let current = guard.take();
                        *guard = Some(get_main_child(current, &config, Arc::clone(&counter)).await);
                    }
                }
                "stop" => {
                    let mut guard = child_state.lock().await;
                    match guard.take() {
                        Some(server) => {
                            let mut server = server.lock().await;
                            stop_bedrock_server(&mut *server).await;
                        }
                        None => println!("[MBH] No running Bedrock Server to stop."),
                    }
                },
                "help" | _ => println!("[MBH] MBH commands: help, start, stop"),
            }
        } else {
            let guard = child_state.lock().await;
            match guard.as_ref() {
                Some(server) => {
                    let mut server = server.lock().await;
                    if let Some(stdin) = server.child.stdin.as_mut() {
                        let mut message = line;
                        message.push('\n');

                        if let Err(e) = stdin.write_all(message.as_bytes()).await {
                            eprintln!("Failed to write to child stdin: {}", e);
                        }
                    }
                }
                None => println!("[MBH] No Bedrock Server running to send input to."),
            }
        }
    }
}