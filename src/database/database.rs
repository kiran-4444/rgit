use anyhow::Result;
use flate2::read::ZlibDecoder;
use predicates::name;
use std::fs::{read, rename, File};
use std::io::{prelude::*, Cursor};

use std::path::PathBuf;

use crate::database::{Blob, Commit, Tree};
use crate::refs::Refs;
use crate::utils::get_root_path;
use crate::{database::Storable, utils::compress_content, utils::hash_content};

use super::tree::FlatTree;

#[derive(Debug)]
pub enum FileMode {
    Regular,
    Executable,
    Directory,
    Unknown,
}

impl From<FileMode> for u32 {
    fn from(mode: FileMode) -> u32 {
        match mode {
            FileMode::Regular => 0o100644,
            FileMode::Executable => 0o100755,
            FileMode::Directory => 0o040000,
            FileMode::Unknown => 0,
        }
    }
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

#[derive(Debug)]
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

pub struct Header {
    pub object_type: ObjectType,
    pub object_size: usize,
}

impl Header {
    pub fn parse(oid: &str, object_store: PathBuf) -> Self {
        let object_path = object_store.join(&oid[0..2]).join(&oid[2..]);
        let data = read(object_path).unwrap();

        let mut decompressed = ZlibDecoder::new(&data[..]);
        let mut buffer = Vec::new();
        decompressed.read_to_end(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let mut header = Vec::new();
        cursor.read_until(b'\0', &mut header).unwrap();

        let header = String::from_utf8(header).unwrap();

        let mut parts = header.split(' ');
        let object_type = parts.next().unwrap();
        let object_type = ObjectType::from_str(object_type);
        let object_size = parts
            .next()
            .unwrap()
            .trim_end_matches('\0')
            .parse::<usize>()
            .unwrap();

        Header {
            object_type,
            object_size,
        }
    }
}

pub struct Content {
    pub header: Header,
    pub body: Vec<u8>,
}

impl Content {
    pub fn parse(oid: &str, object_store: PathBuf) -> Result<Self> {
        let object_path = object_store.join(&oid[0..2]).join(&oid[2..]);

        let data = read(object_path)?;

        let mut decoder = ZlibDecoder::new(&data[..]);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);
        let mut header = Vec::new();
        cursor.read_until(b'\0', &mut header)?;

        let mut body = Vec::new();
        cursor.read_to_end(&mut body)?;

        let header = Header::parse(&oid, object_store);

        Ok(Content { header, body })
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

    pub fn read_object(&self, oid: &str) -> Result<ParsedContent> {
        let header = Header::parse(oid, self.object_store.clone());

        let parsed_content = match header.object_type {
            ObjectType::Blob => ParsedContent::BlobContent(Blob::parse(oid.to_owned())),
            ObjectType::Commit => ParsedContent::CommitContent(Commit::parse(
                oid.to_owned(),
                self.object_store.clone(),
            )),
            ObjectType::Tree => {
                let file_or_dir = Tree::parse(oid.to_owned());
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

    pub fn read_commits(&self) -> Result<Vec<Commit>> {
        let root_part = get_root_path()?;
        let git_path = root_part.join(".rgit");
        let refs = Refs::new(git_path);
        let parent = refs.read_head();

        let mut commits = Vec::new();
        let mut current_oid = parent.unwrap();
        loop {
            let commit = self.read_object(&current_oid).unwrap();
            match commit {
                ParsedContent::CommitContent(commit) => {
                    commits.push(commit.clone());
                    match commit.parent {
                        Some(oid) => current_oid = oid,
                        None => break,
                    }
                }
                _ => panic!("should not happen"),
            }
        }

        Ok(commits)
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

    pub fn prefix_match(&self, prefix: &str) -> Vec<Commit> {
        let commits = self.read_commits().unwrap();

        let mut matched = Vec::new();
        for commit in commits {
            if commit.oid.clone().expect("no OID found").starts_with(prefix) {
                matched.push(commit.clone());
            }
        }

        matched
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
