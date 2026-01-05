use anyhow::Result;
use common::WsMessage;
use rdev::{listen, EventType, Key};
use std::thread;
use tokio::sync::mpsc;
use uuid::Uuid;

pub fn start_global_listener(tx_ws: mpsc::Sender<WsMessage>, my_uuid: Uuid) -> Result<()> {
    // Run rdev listener in a dedicated thread (blocking)
    thread::spawn(move || {
        let mut last_trigger: Option<std::time::Instant> = None;
        if let Err(error) = listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                if key == Key::F9 {
                    let now = std::time::Instant::now();
                    if let Some(last) = last_trigger {
                        if now.duration_since(last) < std::time::Duration::from_secs(10) {
                            println!("Cooldown active (10s). Ignoring F9.");
                            return;
                        }
                    }
                    last_trigger = Some(now);

                    println!("F9 pressed! Sending ring_bell...");
                    let msg = WsMessage::ring_bell(Some(my_uuid.to_string()));
                    let _ = tx_ws.blocking_send(msg);
                }
            }
        }) {
            eprintln!("Error: {:?}", error);
        }
    });
    Ok(())
}
