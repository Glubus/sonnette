use anyhow::Result;
use rand::seq::SliceRandom;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub fn play_random_sound() -> Result<()> {
    // Get output stream handle
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        eprintln!(
            "Assets directory not found at {:?}",
            std::env::current_dir()?.join(assets_dir)
        );
        return Ok(());
    }

    let mut sounds: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(assets_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "mp3" || ext == "wav" {
                    sounds.push(path);
                }
            }
        }
    }

    if sounds.is_empty() {
        println!("No sounds found in assets directory.");
        return Ok(());
    }

    let chosen = sounds.choose(&mut rand::thread_rng()).unwrap();
    println!("Playing: {:?}", chosen);

    let file = BufReader::new(File::open(chosen)?);
    let source = Decoder::new(file)?;

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

pub fn play_sound_by_hash(target_hash: &str) -> Result<()> {
    let hashes = crate::sync::get_local_hashes()?;
    let filename = hashes
        .iter()
        .find(|(_, hash)| *hash == target_hash)
        .map(|(name, _)| name);

    if let Some(filename) = filename {
        let path = Path::new("assets").join(filename);
        println!("Hashes matched! Playing: {:?}", path);

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::new(file)?;
        sink.append(source);
        sink.sleep_until_end();
    } else {
        println!(
            "Hash {} not found locally. Playing random fallback.",
            target_hash
        );
        play_random_sound()?;
    }
    Ok(())
}
