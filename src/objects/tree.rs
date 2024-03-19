use anyhow::Result;
use itertools::Itertools;
use std::{collections::BTreeMap, fs, iter::zip, os::unix::fs::PermissionsExt};

use crate::{database::Database, objects::index::Entry, objects::storable::Storable};

#[derive(Debug, PartialEq, Eq)]
pub enum EntryOrTree {
    Entry(Entry),
    Tree(Tree),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tree {
    pub oid: Option<String>,
    pub entries: BTreeMap<String, EntryOrTree>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            oid: None,
            entries: BTreeMap::new(),
        }
    }

    pub fn build(entries: Vec<Entry>) -> Result<Tree> {
        let mut root = Tree::new();
        for entry in entries {
            root.add_entry(entry.parent_directories()?, EntryOrTree::Entry(entry));
        }
        Ok(root)
    }

    pub fn traverse(&mut self, db: &mut Database) -> Result<()> {
        for (_key, value) in &mut self.entries {
            match value {
                EntryOrTree::Tree(tree) => {
                    tree.traverse(db)?;
                    db.store(tree)?;
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub fn add_entry(&mut self, parents: Vec<String>, entry: EntryOrTree) {
        if parents.is_empty() {
            match entry {
                EntryOrTree::Entry(entry) => {
                    let basename = entry
                        .path
                        .split("/")
                        .last()
                        .expect("Failed to split path to get basename")
                        .to_string();
                    self.entries.insert(basename, EntryOrTree::Entry(entry));
                }
                _ => {
                    panic!("Invalid entry type");
                }
            }
        } else {
            let parent_basename: Vec<String> = parents[0]
                .split("/")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let result = self.entries.get_mut(
                parent_basename
                    .last()
                    .expect("failed to get parent basename")
                    .as_str(),
            );
            match result {
                Some(EntryOrTree::Tree(tree)) => {
                    tree.add_entry(parents[1..].to_vec(), entry);
                }
                _ => {
                    let mut tree = Tree::new();
                    tree.add_entry(parents[1..].to_vec(), entry);
                    self.entries.insert(
                        parent_basename
                            .last()
                            .expect("failed to get parent basename")
                            .to_owned(),
                        EntryOrTree::Tree(tree),
                    );
                }
            }
        }
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
                EntryOrTree::Entry(entry) => {
                    let mut output: Vec<&[u8]> = Vec::new();
                    let entry_path = entry.path.trim_end_matches('\0');
                    let stat = fs::metadata(&entry_path).expect("Failed to get file metadata");
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

                    let decoded = hex::decode(&entry.oid).expect("Failed to decode oid");
                    hex_oids.push(decoded.clone());
                    output
                }
                EntryOrTree::Tree(tree) => {
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
