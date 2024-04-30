use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::{fs, path::PathBuf};

use std::io::BufRead;

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

pub fn get_object_path(oid: &str) -> PathBuf {
    let root_path = get_root_path().expect("Failed to get root path");
    root_path
        .join(".rgit")
        .join("objects")
        .join(&oid[..2])
        .join(&oid[2..])
}

pub fn decompress_content(oid: &str) -> Result<String> {
    let path = get_object_path(oid);
    let data = fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&data[..]);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    let mut cursor = std::io::Cursor::new(buffer);
    let mut header = Vec::new();
    cursor.read_until(b'\0', &mut header)?;
    let mut content = String::new();
    cursor.read_to_string(&mut content)?;
    Ok(content)
}

pub fn get_root_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    if !current_dir.join(".rgit").exists() {
        // anyhow::bail!("fatal: not a git repository (or any of the parent directories): .rgit");
        anyhow::bail!("fatal: not a git repository (or any of the parent directories): .rgit");
    }
    Ok(current_dir)
}
