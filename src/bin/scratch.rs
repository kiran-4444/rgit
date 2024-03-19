use sha1::{Digest, Sha1};

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let hashed_content = hasher.finalize();
    format!("{:x}", hashed_content)
}

fn main() {
    let content = std::fs::read_to_string("Cargo.toml").expect("Failed to read file");
    let hashed_content_string = hash_content(content.as_str());

    let content = std::fs::read("Cargo.toml").expect("Failed to read file");
    let converted = unsafe { std::str::from_utf8_unchecked(&content) };
    let hashed_content_u8 = hash_content(converted);

    assert_eq!(hashed_content_string, hashed_content_u8);
}
