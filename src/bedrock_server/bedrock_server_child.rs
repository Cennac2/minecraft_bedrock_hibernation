use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use tokio::sync::Mutex;

use crate::bedrock_server::bedrock_server_io::{handle_server_error, handle_server_output};
use crate::bedrock_server::bedrock_server_status::{
    get_server_motd, is_bedrock_server_alive, is_bedrock_server_online,
};
use crate::config::config::Config;
use crate::proxy::proxy::PLAYERS_COUNTER;

pub type SharedBedrockServer = Arc<Mutex<Option<Child>>>;

static SERVER_STOPPED_MANUALLY: AtomicBool = AtomicBool::new(false);

fn get_bedrock_server_child(config: Config) -> Result<Child, std::io::Error> {
    let bedrock_path = Path::new(&config.bedrock_file_path)
        .canonicalize()
        .map_err(|e| {
            eprintln!("[MBH] Failed to canonicalize bedrock_server path: {}", e);
            e
        })?;

    let bedrock_dir = bedrock_path.parent().ok_or_else(|| {
        eprintln!("[MBH] bedrock_file_path should have a parent directory");
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bedrock_file_path should have a parent directory",
        )
    })?;

    Command::new(&bedrock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&bedrock_dir)
        .spawn()
        .map_err(|e| {
            eprintln!("[MBH] Failed to start bedrock server: {}", e);
            e
        })
}

pub async fn start_bedrock_server(bedrock_server: SharedBedrockServer, config: Config) {
    {
        let mut guard = bedrock_server.lock().await;
        let (stdout, stderr) = {
            println!("[MBH] Starting up Bedrock Server");

            update_bedrock_server_port(config.clone());

            match get_bedrock_server_child(config.clone()) {
                Ok(child) => {
                    guard.get_or_insert_with(|| child);
                    match guard.as_mut() {
                        Some(child) => (child.stdout.take(), child.stderr.take()),
                        None => {
                            eprintln!("[MBH] no child process is currently running");
                            (None, None)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[MBH] Failed to start bedrock server: {}", e);
                    (None, None)
                }
            }
        };

        SERVER_STOPPED_MANUALLY.store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(child_stdout) = stdout {
            tokio::spawn(handle_server_output(child_stdout));
        }

        if let Some(child_stderr) = stderr {
            tokio::spawn(handle_server_error(child_stderr));
        }
    }

    tokio::spawn(update_server_status(bedrock_server, config));
}

pub async fn start_server_then_get_motd(config: Config) -> Option<String> {
    let bedrock_server = Arc::new(Mutex::new(None));

    {
        let mut guard = bedrock_server.lock().await;
        println!("[MBH] Getting server motd..");

        update_bedrock_server_port(config.clone());

        match get_bedrock_server_child(config.clone()) {
            Ok(child) => {
                *guard = Some(child);
            }
            Err(e) => {
                eprintln!("[MBH] Failed to start bedrock server: {}", e);
                return None;
            }
        }
    }

    let mut motd = None;

    for _ in 1..=20 {
        if is_bedrock_server_online(config.clone()).await {
            motd = get_server_motd(config.clone()).await;
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    stop_bedrock_server(bedrock_server).await;

    motd
}

fn update_bedrock_server_port(config: Config) {
    let bedrock_path = Path::new(&config.bedrock_file_path);

    let bedrock_dir = bedrock_path
        .parent()
        .expect("bedrock_file_path should have a parent directory");

    let path = bedrock_dir.join("server.properties");
    let key = "server-port";
    let new_value = config.bedrock_server_port.to_string();

    let contents = std::fs::read_to_string(path.clone()).unwrap_or_default();
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

async fn update_server_status(server: SharedBedrockServer, config: Config) {
    let mut idle_seconds: u32 = 0;

    loop {
        {
            let mut guard = server.lock().await;

            match guard.as_mut() {
                Some(child) => {
                    if let Ok(Some(status)) = child.try_wait() {
                        if !SERVER_STOPPED_MANUALLY.load(std::sync::atomic::Ordering::SeqCst) {
                            eprintln!("[MBH] Server exited unexpectedly! (exit: {})", status);
                        }
                        *guard = None;
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }

        let active = is_bedrock_server_alive(server.clone()).await;
        let is_online = is_bedrock_server_online(config.clone()).await;

        if active && is_online {
            if PLAYERS_COUNTER.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                idle_seconds += 1;

                if idle_seconds >= config.stop_empty_server_after_seconds {
                    if PLAYERS_COUNTER.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                        println!(
                            "[MBH] No players connected for {} seconds, stopping server..",
                            config.stop_empty_server_after_seconds
                        );
                        stop_bedrock_server(server).await;
                        break;
                    } else {
                        idle_seconds = 0;
                    }
                }
            } else {
                idle_seconds = 0;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn stop_bedrock_server(server: SharedBedrockServer) {
    let mut guard = server.lock().await;

    let Some(child) = guard.as_mut() else {
        return;
    };

    let Some(stdin) = child.stdin.as_mut() else {
        return;
    };

    if let Err(e) = stdin.write_all(b"stop\n").await {
        eprintln!("[MBH] Failed to write to child stdin: {}", e);
        let _ = child.kill();
    } else {
        let _ = stdin.flush().await;
    };

    SERVER_STOPPED_MANUALLY.store(true, std::sync::atomic::Ordering::SeqCst);

    *guard = None;
}
