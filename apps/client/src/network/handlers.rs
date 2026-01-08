use crate::audio::{play_random_sound, play_sound_by_hash};
use crate::sync::save_file;
use common::{FileTransfer, WsMessage};
use uuid::Uuid;

pub fn handle_incoming_message(
    msg: Option<
        Result<
            tokio_tungstenite::tungstenite::protocol::Message,
            tokio_tungstenite::tungstenite::Error,
        >,
    >,
    my_uuid: Uuid,
) -> bool {
    use tokio_tungstenite::tungstenite::protocol::Message;
    match msg {
        Some(Ok(Message::Text(text))) => {
            if let Ok(parsed) = serde_json::from_str::<WsMessage>(&text) {
                dispatch_event(&parsed, my_uuid);
            } else if text.trim() == "ring_bell" {
                println!("Ring bell triggered (legacy)!");
                let _ = play_random_sound();
            }
            true
        }
        Some(Ok(_)) => true,
        Some(Err(e)) => {
            eprintln!("Error receiving message: {}", e);
            false
        }
        None => false,
    }
}

fn dispatch_event(parsed: &WsMessage, my_uuid: Uuid) {
    match parsed.event.as_str() {
        "ring_bell" => handle_ring_bell(parsed, my_uuid),
        "file_transfer" => handle_file_transfer(parsed),
        _ => {}
    }
}

fn handle_ring_bell(parsed: &WsMessage, my_uuid: Uuid) {
    if should_ring(parsed, my_uuid) {
        println!("Ring bell triggered!");

        let mut played_specific = false;
        if let Some(data) = &parsed.data {
            if let Ok(hash) = serde_json::from_value::<String>(data.clone()) {
                println!("Server requested hash: {}", hash);
                if let Err(e) = play_sound_by_hash(&hash) {
                    eprintln!("Failed to play by hash: {}", e);
                } else {
                    played_specific = true;
                }
            }
        }

        if !played_specific {
            println!("No hash provided or failed. Playing random fallback.");
            if let Err(e) = play_random_sound() {
                eprintln!("Failed to play sound: {}", e);
            }
        }

        // Show Notification
        use notify_rust::Notification;
        let _ = Notification::new()
            .summary("Sonnerie")
            .body("🔔 Ding Dong ! On vous appelle !")
            .appname("Sonnerie")
            .show();
    }
}

fn handle_file_transfer(parsed: &WsMessage) {
    if let Some(data) = &parsed.data {
        if let Ok(transfer) = serde_json::from_value::<FileTransfer>(data.clone()) {
            if let Err(e) = save_file(&transfer.filename, &transfer.content) {
                eprintln!("Failed to save file {}: {}", transfer.filename, e);
            }
        }
    }
}

fn should_ring(parsed: &WsMessage, my_uuid: Uuid) -> bool {
    match parsed.sender_id {
        Some(ref id) => id != &my_uuid.to_string(),
        None => true,
    }
}
