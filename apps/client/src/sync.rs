use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const ASSETS_DIR: &str = "assets";

pub fn get_local_hashes() -> Result<HashMap<String, String>> {
    let mut hashes = HashMap::new();
    let path = Path::new(ASSETS_DIR);

    if !path.exists() {
        fs::create_dir_all(path).context("Failed to create assets directory")?;
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                // Determine if audio file
                if filename.ends_with(".mp3") || filename.ends_with(".wav") {
                    let hash = calculate_hash(&path)?;
                    hashes.insert(filename.to_string(), hash);
                }
            }
        }
    }
    Ok(hashes)
}

fn calculate_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub fn save_file(filename: &str, content_base64: &str) -> Result<()> {
    // Security check: simple filename only
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(anyhow::anyhow!("Invalid filename"));
    }

    let decoded = general_purpose::STANDARD.decode(content_base64)?;
    let path = Path::new(ASSETS_DIR).join(filename);

    let mut file = fs::File::create(&path)?;
    file.write_all(&decoded)?;

    println!("Downloaded asset: {}", filename);
    Ok(())
}
