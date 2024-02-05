use std::io::Write;
use std::path::PathBuf;

use crate::blob::Blob;
use crate::utils::compress_content;
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
    pub fn store(&self, blob: &mut Blob) {
        // store the blob in the database
        let content_size = blob.data.len();
        println!("Content size = {content_size}");
        let content = blob.data();
        println!("Content = {content}");
        let content = format!("{} {}\0{}", blob.blob_type(), content_size, content);
        let hashed_content = hash_content(&content);
        blob.set_oid(&hashed_content);
        self.write_object(&hashed_content, &content);
    }

    fn temp_file_path(&self, object_path: &str, name: &str) -> PathBuf {
        PathBuf::from(object_path).join(format!("{}.tmp", name))
    }

    pub fn write_object(&self, name: &str, content: &str) {
        let object_path = PathBuf::from(&self.db_path).join(&name[0..2]);
        std::fs::create_dir_all(&object_path).unwrap();
        let object_name = object_path.join(&name[2..]);
        // generate a temporary file and write the content to it, then rename it to the final nam
        let temp_file_path = self.temp_file_path(
            &object_path.to_str().unwrap(),
            object_name.to_str().unwrap(),
        );
        println!("Writing object to {:?}", temp_file_path);
        let temp_file = std::fs::File::create(&temp_file_path).unwrap();
        let compressed_content = compress_content(content);
        let mut buffer = std::io::BufWriter::new(&temp_file);
        buffer.write_all(&compressed_content).unwrap();
        std::fs::rename(&temp_file_path, &object_name).unwrap();
    }
}
