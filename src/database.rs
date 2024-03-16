use std::io::Write;
use std::path::PathBuf;

use crate::{objects::Storable, utils::compress_content, utils::hash_content};

pub struct Database {
    pub object_store: PathBuf,
}

impl<'a> Database {
    pub fn new(object_store: PathBuf) -> Self {
        Self { object_store }
    }

    pub fn store<T>(&self, storable: &mut T)
    where
        T: Storable,
    {
        // store the storable in the database
        let content = storable.data();
        let content = format!("{} {}\0{}", storable.blob_type(), content.len(), content);
        let hashed_content = hash_content(&content);
        storable.set_oid(hashed_content.to_owned());
        self.write_object(&hashed_content, &content);
    }

    pub fn write_object(&self, name: &str, content: &str) {
        let object_path = PathBuf::from(&self.object_store).join(&name[0..2]);
        std::fs::create_dir_all(&object_path).expect("Failed to create object directory");
        let object_name = object_path.join(&name[2..]);

        // if the object already exists, we don't need to write it again
        if object_name.exists() {
            return;
        }

        // generate a temporary file and write the content to it, then rename it to the final name
        let temp_file_path =
            PathBuf::from(object_path).join(format!("{}.tmp", object_name.display()));
        let temp_file = std::fs::File::create(&temp_file_path).expect("Failed to create temp file");
        let compressed_content = compress_content(content);
        let mut buffer = std::io::BufWriter::new(&temp_file);
        buffer
            .write_all(&compressed_content)
            .expect("Failed to write to temp file");
        std::fs::rename(&temp_file_path, &object_name)
            .expect("failed to rename the temp file to object name");
    }
}
