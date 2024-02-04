use sha1::{Digest, Sha1};

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let hashed_content = hasher.finalize();

    format!("{:x}", hashed_content)
}
