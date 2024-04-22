use anyhow::Result;
use itertools::Itertools;
use std::{collections::BTreeMap, fs, iter::zip, os::unix::fs::PermissionsExt};

use crate::{
    database::{storable::Storable, Database},
    index::Index,
    workspace::{Dir, File, FileOrDir},
};

#[derive(Debug, Clone, PartialEq)]
pub enum FileOrTree {
    File(File),
    Tree(Tree),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tree {
    pub oid: Option<String>,
    pub entries: BTreeMap<String, FileOrTree>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            oid: None,
            entries: BTreeMap::new(),
        }
    }

    pub fn build(&mut self, dir: Dir) {
        for (path, entry) in &dir.children {
            match entry {
                FileOrDir::File(file) => {
                    self.entries
                        .insert(path.to_owned(), FileOrTree::File(file.clone()));
                }
                FileOrDir::Dir(dir) => {
                    let mut tree = Tree::new();
                    tree.build(dir.clone());
                    self.entries.insert(path.to_owned(), FileOrTree::Tree(tree));
                }
            }
        }
    }

    pub fn build_from_index(&mut self, index: &Index) {
        for (path, entry) in &index.entries {
            match entry {
                FileOrDir::File(file) => {
                    self.entries
                        .insert(path.to_owned(), FileOrTree::File(file.clone()));
                }
                FileOrDir::Dir(dir) => {
                    let mut tree = Tree::new();
                    tree.build(dir.clone());
                    self.entries.insert(path.to_owned(), FileOrTree::Tree(tree));
                }
            }
        }
    }

    pub fn traverse(&mut self, db: &mut Database) -> Result<()> {
        for (_, value) in &mut self.entries {
            match value {
                FileOrTree::Tree(tree) => {
                    tree.traverse(db)?;
                    db.store(tree)?;
                }
                _ => (),
            }
        }

        Ok(())
    }
}

impl Storable for Tree {
    fn blob_type(&self) -> String {
        "tree".to_owned()
    }

    fn set_oid(&mut self, oid: String) {
        self.oid = Some(oid);
    }

    fn data(&self) -> String {
        let mut hex_oids: Vec<Vec<u8>> = Vec::new();
        let mut entries = self
            .entries
            .iter()
            .map(|(name, entry)| match entry {
                FileOrTree::File(entry) => {
                    let mut output: Vec<&[u8]> = Vec::new();

                    let stat = fs::metadata(&entry.path).expect("Failed to get file metadata");
                    let is_executable = stat.permissions().mode() & 0o111 != 0;
                    if is_executable {
                        output.push("100755".as_bytes());
                    } else {
                        output.push("100644".as_bytes());
                    };
                    output.push(&[b' ']);

                    let entry_name_bytes = name.as_bytes();
                    output.push(entry_name_bytes);

                    let null_byte_array = &[b'\x00'];
                    output.push(null_byte_array);

                    let decoded = hex::decode(entry.oid.clone().expect("failed to get oid"))
                        .expect("Failed to decode oid");
                    hex_oids.push(decoded.clone());
                    output
                }
                FileOrTree::Tree(tree) => {
                    let mut output: Vec<&[u8]> = Vec::new();

                    output.push("40000".as_bytes());
                    output.push(&[b' ']);

                    let entry_name_bytes = name.as_bytes();
                    output.push(entry_name_bytes);

                    let null_byte_array = &[b'\x00'];
                    output.push(null_byte_array);

                    let decoded = hex::decode(&tree.oid.as_ref().expect("failed to get tree oid"))
                        .expect("Failed to decode oid");
                    hex_oids.push(decoded.clone());
                    output
                }
            })
            .collect_vec();

        // concatenate the entries
        let mut concatenated_entries: Vec<u8> = Vec::new();
        for (entry, hex_oid) in zip(&mut entries, &hex_oids) {
            entry.push(&hex_oid);
            for e in entry.clone() {
                concatenated_entries.extend(e);
            }
        }
        unsafe { String::from_utf8_unchecked(concatenated_entries) }
    }
}
