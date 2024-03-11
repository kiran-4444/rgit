use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::prelude::*;

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let hashed_content = hasher.finalize();
    format!("{:x}", hashed_content)
}

pub fn compress_content(content: &str) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

pub fn ignored_files() -> Vec<String> {
    let ignores = vec![
        ".",
        "..",
        ".rgit",
        ".git",
        ".pgit",
        "pgit.py",
        ".mypy_cache",
    ];

    ignores.iter().map(|s| s.to_string()).collect()
}

pub fn list_files() -> Result<Vec<String>, std::io::Error> {
    let entries = std::fs::read_dir(".")?;
    let ignore = ignored_files();
    // iterate through the files in the current directory by skipping the IGNORE files or directories
    let files: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let file_name = e.file_name().to_string_lossy().into_owned();
                if !ignore.contains(&file_name) {
                    Some(file_name)
                } else {
                    None
                }
            })
        })
        .collect();
    Ok(files)
}
