use anyhow::Result;
use std::fs::{rename, File};
use std::io::prelude::*;
use std::path::PathBuf;

use crate::{database::Storable, utils::compress_content, utils::hash_content};

pub struct Database {
    pub object_store: PathBuf,
}

impl<'a> Database {
    pub fn new(object_store: PathBuf) -> Self {
        Self { object_store }
    }

    pub fn store<T>(&self, storable: &mut T) -> Result<()>
    where
        T: Storable,
    {
        // store the storable in the database
        let content = storable.data();
        let content = format!("{} {}\0{}", storable.blob_type(), content.len(), content);
        let hashed_content = hash_content(&content);
        storable.set_oid(hashed_content.to_owned());
        self.write_object(&hashed_content, &content)?;
        Ok(())
    }

    pub fn write_object(&self, name: &str, content: &str) -> Result<()> {
        let object_path = PathBuf::from(&self.object_store).join(&name[0..2]);
        std::fs::create_dir_all(&object_path)?;
        let object_name = object_path.join(&name[2..]);

        // if the object already exists, we don't need to write it again
        if object_name.exists() {
            return Ok(());
        }

        // generate a temporary file and write the content to it, then rename it to the final name
        let temp_file_path =
            PathBuf::from(object_path).join(format!("{}.tmp", object_name.display()));
        let temp_file = File::create(&temp_file_path)?;
        let compressed_content = compress_content(content);
        let mut buffer = std::io::BufWriter::new(&temp_file);
        buffer.write_all(&compressed_content)?;
        rename(&temp_file_path, &object_name)?;

        Ok(())
    }
}
