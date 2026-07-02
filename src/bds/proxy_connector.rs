use std::sync::Arc;

use rust_raknet::{RaknetSocket, Reliability};
use tokio::sync::Mutex;

pub async fn proxy_connection(
    client: RaknetSocket,
    server: RaknetSocket,
    counter: Arc<Mutex<u32>>
) {
    {
        let mut client_amount = counter.lock().await;
        *client_amount += 1;
    }

    let client_to_server = async {
        loop {
            let packet = match client.recv().await {
                Ok(p) => p,
                Err(_) => break,
            };

            if server
                .send(&packet, Reliability::ReliableOrdered)
                .await
                .is_err()
            {
                break;
            }
        }
    };

    let server_to_client = async {
        loop {
            let packet = match server.recv().await {
                Ok(p) => p,
                Err(_) => break,
            };

            if client
                .send(&packet, Reliability::ReliableOrdered)
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

    println!("[MBH] {} has left the server!", client.peer_addr().unwrap());

    {
        let mut client_amount = counter.lock().await;
        *client_amount -= 1;
    }
}