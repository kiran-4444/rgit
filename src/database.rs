use crate::blob::Blob;
use crate::utils::hash_content;

pub struct Database {
    pub db_path: String,
}

impl Database {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }
    pub fn store(&self, blob: Blob) {
        // store the blob in the database
        let content_size = blob.data.len();
        println!("Content size = {content_size}");
        let content = blob.data();
        println!("Content = {content}");
        let content = format!("{} {}\0{}", blob.blob_type(), content_size, content);
        let hashed_content = hash_content(&content);
        self.write_object(&hashed_content, &content);
    }

    pub fn write_object(&self, name: &str, content: &str) {
        println!("Name = {name} Content = {content}");
    }
}
