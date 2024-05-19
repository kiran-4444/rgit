use anyhow::Result;
use colored::ColoredString;
use flate2::{write::ZlibEncoder, Compression};
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

pub fn write_to_stdout_color(content: &ColoredString) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
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
        // anyhow::bail!("fatal: not a git repository (or any of the parent directories): .rgit");
        anyhow::bail!("fatal: not a git repository (or any of the parent directories): .rgit");
    }
    Ok(current_dir)
}
