use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::path::PathBuf;

pub fn write_to_stdout(content: &str) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", content)?;
    Ok(())
}

pub fn write_to_stderr(content: &str) -> Result<()> {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    writeln!(handle, "{}", content)?;
    Ok(())
}

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let hashed_content = hasher.finalize();
    format!("{:x}", hashed_content)
}

pub fn compress_content(content: &str) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(content.as_bytes())
        .expect("Failed to compress content");
    encoder.finish().expect("Failed to finish compression")
}

pub fn get_root_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    if !current_dir.join(".rgit").exists() {
        anyhow::bail!("Not a rgit repository");
    }
    Ok(current_dir)
}
