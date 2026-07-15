use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};

use crate::{
    bedrock_server::{
        bedrock_server_child::{SharedBedrockServer, start_bedrock_server, stop_bedrock_server},
        bedrock_server_status::is_bedrock_server_alive,
    },
    config::config::Config,
};

pub async fn handle_server_output(stdout: ChildStdout) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        println!("BDS > {line}");
    }
}

pub async fn handle_server_error(stderr: ChildStderr) {
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        eprintln!("BDS Error > {line}");
    }
}

pub async fn handle_user_input(server: SharedBedrockServer, config: Config) {
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.starts_with("mbh") {
            let arg = line.trim_start_matches("mbh").trim();

            match arg {
                "start" => {
                    let active = is_bedrock_server_alive(server.clone()).await;

                    if active {
                        println!("[MBH] Bedrock Server is already active!");
                    } else {
                        start_bedrock_server(server.clone(), config.clone()).await;
                    }
                }
                "stop" => {
                    let active = is_bedrock_server_alive(server.clone()).await;

                    if !active {
                        println!("[MBH] Bedrock Server is offline.");
                    } else {
                        stop_bedrock_server(server.clone()).await;
                    }
                }
                "exit" => {
                    let active = is_bedrock_server_alive(server.clone()).await;

                    if active {
                        stop_bedrock_server(server).await;
                    }

                    tokio::time::sleep(Duration::from_secs(2)).await; // wait for minecraft itself to stop, idk another way to wait for it other than this but it works.

                    println!("[MBH] Shutdown complete.");
                    std::process::exit(0);
                }
                "help" | _ => println!("[MBH] MBH commands: help, start, stop"),
            }
        } else {
            let mut guard = server.lock().await;

            if let Some(stdin) = guard.as_mut().and_then(|child| child.stdin.as_mut()) {
                let mut message = line;
                message.push('\n');

                if let Err(e) = stdin.write_all(message.as_bytes()).await {
                    eprintln!("[MBH] Failed to write to child stdin: {}", e);
                }
            } else {
                println!(
                    "[MBH] Bedrock Server is not online! run 'mbh start' or join to start it up!"
                );
            };
        }
    }
}
