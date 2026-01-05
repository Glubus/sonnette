use common::{SyncRequest, WsMessage};
use tokio::sync::mpsc;

pub async fn handle_sync(request: SyncRequest, local_tx: &mpsc::Sender<String>) {
    let server_hashes = crate::sync::get_server_hashes().await.unwrap_or_default();

    for (filename, server_hash) in server_hashes {
        let client_hash = request.hashes.get(&filename);
        if client_hash != Some(&server_hash) {
            tracing::info!("Client needs update for {}", filename);
            // Send file via local channel
            if let Ok(content) = crate::sync::read_file_content(&filename).await {
                let msg = WsMessage::file_transfer(filename, content);
                let json = serde_json::to_string(&msg).unwrap();
                let _ = local_tx.send(json).await;
            }
        }
    }
}
