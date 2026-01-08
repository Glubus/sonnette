mod audio;
mod input;
mod network;
mod sync;

use crate::input::start_global_listener;
use crate::network::run_ws_client;
use common::WsMessage;
use tao::event_loop::{ControlFlow, EventLoop};
use tao::platform::run_return::EventLoopExtRunReturn; // Helper if needed, but standard run is usually fine
use tokio::sync::mpsc;
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use uuid::Uuid;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    println!("Starting Sonnerie Client (Tray Mode)");

    let event_loop = EventLoop::new();

    // -- System Tray Setup --
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quitter", true, None);
    tray_menu.append(&quit_i).unwrap();

    let mut _tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Sonnerie Client")
            .with_icon(load_icon())
            .build()
            .unwrap(),
    );

    let my_uuid = Uuid::new_v4();
    println!("My Client UUID: {}", my_uuid);

    // Channel for input thread -> async ws task
    let (tx, rx) = mpsc::channel::<WsMessage>(100);

    // -- Start Logic Threads --

    // 1. Global Input Listener (Blocking)
    // We clone tx because start_global_listener takes ownership or needs a clone
    let tx_clone = tx.clone();
    // Note: start_global_listener spawns its own thread internally, so we just call it.
    if let Err(e) = start_global_listener(tx_clone, my_uuid) {
        eprintln!("Failed to start global listener: {}", e);
    }

    // 2. WebSocket/Async Runtime (Blocking thread)
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(run_ws_client(my_uuid, rx));
    });

    // -- Run Event Loop (Main Thread) --
    // We must run this on the main thread for UI/Tray compatibility
    let menu_channel = tray_icon::menu::MenuEvent::receiver();
    // let tray_channel = tray_icon::TrayIconEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                // cleanup
                _tray_icon.take();
                *control_flow = ControlFlow::Exit;
                std::process::exit(0);
            }
        }
    });
}

fn load_icon() -> tray_icon::Icon {
    // Generate a simple 32x32 green/red icon manually since we might not have a file handy immediately
    // or use a helper to load from file. For now, generated pixel buffer.
    let (icon_rgba, icon_width, icon_height) = {
        let width = 32;
        let height = 32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for i in 0..width * height {
            // Greenish color
            rgba.push(0);
            rgba.push(255);
            rgba.push(0);
            rgba.push(255);
        }
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
