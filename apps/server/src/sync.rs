use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

const ASSETS_DIR: &str = "assets";

pub async fn get_server_hashes() -> std::io::Result<HashMap<String, String>> {
    let mut hashes = HashMap::new();
    let path = Path::new(ASSETS_DIR);

    if !path.exists() {
        fs::create_dir_all(path).await?;
        return Ok(hashes);
    }

    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.ends_with(".mp3") || filename.ends_with(".wav") {
                    if let Ok(hash) = calculate_hash(&path).await {
                        hashes.insert(filename.to_string(), hash);
                    }
                }
            }
        }
    }
    Ok(hashes)
}

async fn calculate_hash(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub async fn read_file_content(filename: &str) -> std::io::Result<String> {
    // Security check
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid filename",
        ));
    }

    let path = Path::new(ASSETS_DIR).join(filename);
    let content = fs::read(&path).await?;
    Ok(general_purpose::STANDARD.encode(content))
}
