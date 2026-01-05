use crate::state::AppState;
use common::WsMessage;
use rand::seq::SliceRandom;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn handle_ring_bell(
    sender_id: Option<String>,
    state: &Arc<AppState>,
    tx: &broadcast::Sender<String>,
) {
    // 0. Cooldown check
    if let Some(id) = &sender_id {
        let mut last_trigger = state.last_trigger.lock().unwrap();
        let now = std::time::Instant::now();
        if let Some(last) = last_trigger.get(id) {
            if now.duration_since(*last) < std::time::Duration::from_secs(10) {
                tracing::warn!("Cooldown active for user {}, ignoring ring.", id);
                return;
            }
        }
        last_trigger.insert(id.clone(), now);
    }

    // 1. Get all server hashes
    let hashes_map = crate::sync::get_server_hashes().await.unwrap_or_default();

    // 2. Pick a random hash if available
    let values: Vec<&String> = hashes_map.values().collect();
    let mut chosen_hash: Option<String> = None;

    if let Some(hash) = values.choose(&mut rand::thread_rng()) {
        chosen_hash = Some(hash.to_string());
        tracing::info!("Server selected sound hash: {}", hash);
    } else {
        tracing::warn!("No assets found on server, sending empty hash");
    }

    // 3. Construct message
    let mut msg = WsMessage::ring_bell(sender_id);
    // Overwrite data with the hash string (as JSON string)
    if let Some(h) = chosen_hash {
        msg.data = Some(serde_json::to_value(h).unwrap());
    }

    // 4. Broadcast
    let text = serde_json::to_string(&msg).unwrap();
    let _ = tx.send(text);
}
