use anyhow::Result;
use std::fs::{rename, File};
use std::io::prelude::*;
use std::path::PathBuf;

use crate::database::{Blob, Commit, Tree};
use crate::refs::Refs;
use crate::utils::get_root_path;
use crate::{database::Storable, utils::compress_content, utils::hash_content};

use super::tree::FlatTree;

pub enum FileMode {
    Regular,
    Executable,
    Directory,
    Unknown,
}

impl FileMode {
    pub fn from_str(mode: &str) -> Self {
        match mode {
            "100644" => FileMode::Regular,
            "100755" => FileMode::Executable,
            "040000" => FileMode::Directory,
            _ => FileMode::Unknown,
        }
    }
}

pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Unknown,
}

impl ObjectType {
    pub fn from_str(object_type: &str) -> Self {
        match object_type {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            "commit" => ObjectType::Commit,
            _ => ObjectType::Unknown,
        }
    }
}

pub struct Database {
    pub object_store: PathBuf,
    pub objects: Vec<String>,
}

#[derive(Debug)]
pub enum ParsedContent {
    BlobContent(Blob),
    CommitContent(Commit),
    TreeContent(FlatTree),
}

impl<'a> Database {
    pub fn new(object_store: PathBuf) -> Self {
        Self {
            object_store,
            objects: Default::default(),
        }
    }

    fn object_path(&self, name: &str) -> PathBuf {
        PathBuf::from(&self.object_store)
            .join(&name[0..2])
            .join(&name[2..])
    }

    pub fn read_object(&self, oid: &str) -> Result<ParsedContent> {
        let data = std::fs::read(self.object_path(oid))?;
        let mut decoder = flate2::read::ZlibDecoder::new(&data[..]);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        let mut cursor = std::io::Cursor::new(buffer);
        let mut header = Vec::new();
        cursor.read_until(b'\0', &mut header)?;
        let mut content = Vec::new();
        cursor.read_to_end(&mut content)?;

        let mut header_cursor = std::io::Cursor::new(header);
        let mut object_type = Vec::new();
        header_cursor.read_until(b' ', &mut object_type)?;
        let mut object_size = Vec::new();
        header_cursor.read_until(b'\0', &mut object_size)?;

        let object_type = String::from_utf8(object_type)?;
        let object_type = object_type.trim_end_matches(' ');
        let object_type = ObjectType::from_str(&object_type);

        let object_size = String::from_utf8(object_size)?;
        let _object_size = object_size.trim_end_matches('\0').parse::<usize>()?;

        let parsed_content = match object_type {
            ObjectType::Blob => ParsedContent::BlobContent(Blob::parse(oid.to_owned(), content)),
            ObjectType::Commit => {
                ParsedContent::CommitContent(Commit::parse(oid.to_owned(), content))
            }
            ObjectType::Tree => {
                let file_or_dir = Tree::parse(content, None);
                ParsedContent::TreeContent(FlatTree {
                    entries: file_or_dir,
                })
            }

            ObjectType::Unknown => {
                panic!("Unknown object type");
            }
        };

        Ok(parsed_content)
    }

    pub fn read_head(&self) -> Result<FlatTree> {
        let root_part = get_root_path()?;
        let git_path = root_part.join(".rgit");
        let refs = Refs::new(git_path);
        let parent = refs.read_head();

        let tree = match parent {
            Some(oid) => {
                let commit = self.read_object(&oid).unwrap();
                match commit {
                    ParsedContent::CommitContent(commit) => {
                        let tree_oid = commit.tree;
                        let tree = self.read_object(&tree_oid).unwrap();
                        match tree {
                            ParsedContent::TreeContent(tree) => tree,
                            _ => panic!("should not happen"),
                        }
                    }
                    _ => panic!("should not happen"),
                }
            }
            None => FlatTree {
                entries: Default::default(),
            },
        };

        Ok(tree)
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
