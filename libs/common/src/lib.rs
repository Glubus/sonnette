use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsMessage {
    pub event: String,
    pub sender_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl WsMessage {
    pub fn ring_bell(sender_id: Option<String>) -> Self {
        Self {
            event: "ring_bell".to_string(),
            sender_id,
            data: None,
        }
    }

    pub fn sync_hashes(hashes: HashMap<String, String>) -> Self {
        Self {
            event: "sync_hashes".to_string(),
            sender_id: None,
            data: Some(serde_json::to_value(SyncRequest { hashes }).unwrap()),
        }
    }

    pub fn file_transfer(filename: String, content: String) -> Self {
        Self {
            event: "file_transfer".to_string(),
            sender_id: None,
            data: Some(serde_json::to_value(FileTransfer { filename, content }).unwrap()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncRequest {
    pub hashes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileTransfer {
    pub filename: String,
    pub content: String,
}
