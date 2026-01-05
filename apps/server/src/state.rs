use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::broadcast;

pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub last_trigger: Mutex<HashMap<String, Instant>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(100);
        Arc::new(Self {
            tx,
            last_trigger: Mutex::new(HashMap::new()),
        })
    }
}
