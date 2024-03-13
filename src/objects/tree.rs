use std::{
    collections::{BTreeMap, HashMap},
    iter::zip,
    path::Path,
};

use itertools::Itertools;

use super::{storable::Storable, Entry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryOrTree {
    Entry(Entry),
    Tree(Tree),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub oid: Option<String>,
}

impl Tree {
    pub fn new() -> Self {
        Self { oid: None }
    }

    pub fn build(entries: Vec<Entry>) -> Tree {
        let root = Tree::new();
        let mut hash_entries: BTreeMap<String, EntryOrTree> = BTreeMap::new();
        for entry in entries {
            root.add_entry(
                entry.parent_directories(),
                EntryOrTree::Entry(entry),
                &mut hash_entries,
            );
        }

        // for (key, value) in hash_entries.iter() {
        //     dbg!(key);
        //     dbg!(value);

        //     match value {
        //         EntryOrTree::Entry(entry) => {
        //             dbg!(entry);
        //         }
        //         EntryOrTree::Tree(tree) => {
        //             dbg!(tree);
        //         }
        //     }
        // }
        dbg!(hash_entries.clone());
        root
    }

    pub fn add_entry(
        &self,
        parents: Vec<String>,
        entry: EntryOrTree,
        tree_entries: &mut BTreeMap<String, EntryOrTree>,
    ) {
        if parents.is_empty() {
            match entry {
                EntryOrTree::Entry(entry) => {
                    let basename = entry.name.split("/").last().unwrap().to_string();
                    tree_entries.insert(basename, EntryOrTree::Entry(entry));
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
            let tree_entry_copy = tree_entries.clone();
            let result = tree_entry_copy.get(parent_basename.last().unwrap().as_str());
            match result {
                Some(EntryOrTree::Tree(tree)) => {
                    tree.add_entry(parents[1..].to_vec(), entry.clone(), tree_entries);
                }
                _ => {
                    let tree = Tree::new();
                    tree.add_entry(parents[1..].to_vec(), entry.clone(), tree_entries);
                    tree_entries.insert(
                        parent_basename.last().unwrap().to_owned(),
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
        let mut entries_vec: Vec<Entry> = Vec::new();
        entries_vec.sort_by(|a, b| a.name.cmp(&b.name));

        let mut hex_oids: Vec<Vec<u8>> = Vec::new();
        let mut entries = entries_vec
            .iter()
            .map(|entry| {
                let mut output: Vec<&[u8]> = Vec::new();

                output.push(entry.mode.as_bytes());
                output.push(&[b' ']);

                let entry_name_bytes = entry.name.as_bytes();
                output.push(entry_name_bytes);

                let null_byte_array = &[b'\x00'];
                output.push(null_byte_array);

                let decoded = hex::decode(&entry.oid).unwrap();
                hex_oids.push(decoded.clone());
                output
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
