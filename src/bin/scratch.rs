use std::fs::File;
use std::io::{self, Read};

/// Checks if a file is binary by reading the first few bytes and analyzing them.
fn is_binary_file(file_path: &str) -> io::Result<bool> {
    // Open the file
    let mut file = File::open(file_path)?;

    // Read the first 1024 bytes (or less if the file is smaller)
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer)?;

    // Check if the bytes are valid UTF-8
    let content = &buffer[..n];
    Ok(!is_printable(content))
}

/// Checks if a byte slice is printable (i.e., contains valid UTF-8 and printable characters)
fn is_printable(content: &[u8]) -> bool {
    if let Ok(text) = std::str::from_utf8(content) {
        for ch in text.chars() {
            if !ch.is_control() || ch.is_whitespace() {
                continue;
            }
            return false;
        }
        true
    } else {
        false
    }
}

fn main() -> io::Result<()> {
    let file_path = "/Users/luna/Documents/rust_learning/rgit/target/debug/r_git";
    match is_binary_file(file_path)? {
        true => println!("The file is binary."),
        false => println!("The file is text."),
    }
    Ok(())
}
