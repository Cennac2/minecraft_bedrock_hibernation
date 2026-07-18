use rust_raknet::RaknetSocket;

use crate::proxy::proxy::PLAYERS_COUNTER;

pub async fn start_proxy_connection(client: RaknetSocket, server_client: RaknetSocket) {
    PLAYERS_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let client_to_server = async {
        loop {
            let packet = match client.recv().await {
                Ok(p) => p,
                Err(_) => break,
            };

            if server_client
                .send(&packet, rust_raknet::Reliability::ReliableOrdered)
                .await
                .is_err()
            {
                break;
            }
        }
    };

    let server_to_client = async {
        loop {
            let packet = match server_client.recv().await {
                Ok(p) => p,
                Err(_) => break,
            };

            if client
                .send(&packet, rust_raknet::Reliability::ReliableOrdered)
                .await
                .is_err()
            {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_server => {}
        _ = server_to_client => {}
    }

    let peer_addr = client
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("[MBH] {} has left the server!", peer_addr);

    PLAYERS_COUNTER.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
}
