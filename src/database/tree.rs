use anyhow::Result;
use itertools::Itertools;
use std::{collections::BTreeMap, fs, iter::zip, os::unix::fs::PermissionsExt};

use crate::{
    database::{storable::Storable, Database},
    index::Index,
    workspace::{File, FileOrDir},
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

    pub fn build(&mut self, index: &Index) {
        let mut tree = Tree::new();
        for (path, entry) in &index.entries.workspace {
            match entry {
                FileOrDir::File(file) => {
                    let path = path.clone();
                    tree.entries.insert(path, FileOrTree::File(file.clone()));
                }
                FileOrDir::Dir(dir) => {}
            }
        }
    }

    pub fn traverse(&mut self, db: &mut Database) -> Result<()> {
        // for (key, value) in &mut self.entries {
        //     match value {
        //         FileOrDir::Dir(dir) => {
        //             tree.traverse(db)?;
        //             db.store(tree)?;
        //         }
        //         _ => (),
        //     }
        // }

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
        // let mut hex_oids: Vec<Vec<u8>> = Vec::new();
        // let mut entries = self
        //     .entries
        //     .iter()
        //     .map(|(name, entry)| match entry {
        //         EntryOrTree::Entry(entry) => {
        //             let mut output: Vec<&[u8]> = Vec::new();
        //             // let entry_path = entry.path.trim_end_matches('\0');
        //             let entry_path = entry
        //                 .path
        //                 .as_os_str()
        //                 .to_str()
        //                 .expect("failed to convert path to str");
        //             let stat = fs::metadata(&entry_path).expect("Failed to get file metadata");
        //             let is_executable = stat.permissions().mode() & 0o111 != 0;
        //             if is_executable {
        //                 output.push("100755".as_bytes());
        //             } else {
        //                 output.push("100644".as_bytes());
        //             };
        //             output.push(&[b' ']);

        //             let entry_name_bytes = name.as_bytes();
        //             output.push(entry_name_bytes);

        //             let null_byte_array = &[b'\x00'];
        //             output.push(null_byte_array);

        //             let decoded = hex::decode(&entry.oid).expect("Failed to decode oid");
        //             hex_oids.push(decoded.clone());
        //             output
        //         }
        //         EntryOrTree::Tree(tree) => {
        //             let mut output: Vec<&[u8]> = Vec::new();

        //             output.push("40000".as_bytes());
        //             output.push(&[b' ']);

        //             let entry_name_bytes = name.as_bytes();
        //             output.push(entry_name_bytes);

        //             let null_byte_array = &[b'\x00'];
        //             output.push(null_byte_array);

        //             let decoded = hex::decode(&tree.oid.as_ref().expect("failed to get tree oid"))
        //                 .expect("Failed to decode oid");
        //             hex_oids.push(decoded.clone());
        //             output
        //         }
        //     })
        //     .collect_vec();

        // // concatenate the entries
        // let mut concatenated_entries: Vec<u8> = Vec::new();
        // for (entry, hex_oid) in zip(&mut entries, &hex_oids) {
        //     entry.push(&hex_oid);
        //     for e in entry.clone() {
        //         concatenated_entries.extend(e);
        //     }
        // }
        // unsafe { String::from_utf8_unchecked(concatenated_entries) }
        "".to_owned()
    }
}
