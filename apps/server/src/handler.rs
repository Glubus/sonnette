use crate::commands;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use common::WsMessage;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();
    let tx = state.tx.clone();

    // Local channel for sending unicast messages to this client
    let (local_tx, mut local_rx) = mpsc::channel::<String>(100);

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(parsed) => {
                        if parsed.event == "ring_bell" {
                            tracing::info!("Received ring_bell, broadcasting...");
                            commands::ring_bell::handle_ring_bell(parsed.sender_id, &state, &tx)
                                .await;
                        } else if parsed.event == "sync_hashes" {
                            tracing::info!("Received sync_hashes, checking diff...");
                            if let Some(data) = parsed.data {
                                if let Ok(request) =
                                    serde_json::from_value::<common::SyncRequest>(data)
                                {
                                    commands::sync::handle_sync(request, &local_tx).await;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Fallback for raw string
                        if text.trim() == "ring_bell" {
                            tracing::info!("Received raw ring_bell, broadcasting...");
                            let msg = WsMessage::ring_bell(None);
                            commands::ring_bell::handle_ring_bell(msg.sender_id, &state, &tx).await;
                        }
                    }
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Broadcast messages
                Ok(msg) = rx.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                // Local unicast messages (e.g. file transfers)
                Some(msg) = local_rx.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };
}
