pub mod handlers;

use crate::sync::get_local_hashes;
use anyhow::{Context, Result};
use common::WsMessage;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use uuid::Uuid;

// Use handlers
use handlers::handle_incoming_message;

type WsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

const SERVER_URL: &str = "ws://51.254.128.175:3000/ws";

pub async fn run_ws_client(my_uuid: Uuid, mut rx_input: mpsc::Receiver<WsMessage>) {
    let url = Url::parse(SERVER_URL).expect("Invalid URL");

    loop {
        println!("Connecting to {}...", url);
        match connect_async(url.clone()).await {
            Ok((ws_stream, _)) => {
                println!("Connected to WebSocket server with ID: {}", my_uuid);
                let (mut write, read) = ws_stream.split();

                if let Err(e) = send_sync_hashes(&mut write).await {
                    eprintln!("Failed to send sync hashes: {}", e);
                }

                run_interaction_loop(my_uuid, write, read, &mut rx_input).await;
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
            }
        }
        println!("Disconnected, retrying in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn send_sync_hashes(write: &mut WsSender) -> Result<()> {
    let hashes = get_local_hashes().context("Failed to get local hashes")?;
    let msg = WsMessage::sync_hashes(hashes);
    let text = serde_json::to_string(&msg)?;
    write
        .send(Message::Text(text.into()))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("Sent audio file hashes for synchronization.");
    Ok(())
}

async fn run_interaction_loop(
    my_uuid: Uuid,
    mut write: WsSender,
    mut read: WsReceiver,
    rx_input: &mut mpsc::Receiver<WsMessage>,
) {
    loop {
        tokio::select! {
            some_msg = read.next() => {
                if !handle_incoming_message(some_msg, my_uuid) {
                    break;
                }
            }
            Some(msg) = rx_input.recv() => {
                if !send_message(&mut write, msg).await {
                    break;
                }
            }
        }
    }
}

async fn send_message(write: &mut WsSender, msg: WsMessage) -> bool {
    let text = serde_json::to_string(&msg).unwrap();
    if let Err(e) = write.send(Message::Text(text.into())).await {
        eprintln!("Failed to send message: {}", e);
        return false;
    }
    true
}
