mod audio;
mod input;
mod network;
mod sync;

use crate::input::start_global_listener;
use crate::network::run_ws_client;
use common::WsMessage;
use tokio::sync::mpsc;
use uuid::Uuid;

fn main() {
    env_logger::init();

    println!("Starting Sonnerie Client (Background Mode)");
    println!("Note: System Tray support disabled due to missing 'libxdo-dev'.");
    println!("Global F9 trigger enabled.");
    println!("Sync enabled: Connecting to 51.254.128.175...");
    println!("Press Ctrl+C to quit.");

    let my_uuid = Uuid::new_v4();
    println!("My Client UUID: {}", my_uuid);

    // Channel for input thread -> async ws task
    let (tx, rx) = mpsc::channel::<WsMessage>(100);

    // Start global input listener
    if let Err(e) = start_global_listener(tx, my_uuid) {
        eprintln!("Failed to start global listener: {}", e);
    }

    // Run the async websocket client
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_ws_client(my_uuid, rx));
}
