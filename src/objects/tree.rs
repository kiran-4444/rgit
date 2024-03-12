use std::{collections::HashMap, iter::zip};

use itertools::Itertools;

use super::{storable::Storable, Entry};

#[derive(Debug)]
pub struct Tree {
    pub mode: Option<String>,
    pub oid: Option<String>,
    pub entries: Vec<Entry>,
}

impl Tree {
    pub fn new(entries: Vec<Entry>) -> Self {
        if entries.is_empty() {
            Self {
                mode: None,
                oid: None,
                entries: Vec::new(),
            }
        } else {
            Self {
                mode: None,
                oid: None,
                entries,
            }
        }
    }

    pub fn build(entries: Vec<Entry>) -> Vec<(Vec<String>, Entry)> {
        // sort the entries by their name, that's how git does it
        let entries_vec: Vec<Entry> = entries.clone();
        // entries_vec.sort_by(|a, b| a.name.cmp(&b.name));

        entries_vec
            .iter()
            .map(|entry| (entry.parent_directories(), entry.clone()))
            .collect()
    }

    pub fn _add_entry(
        parents: Vec<String>,
        entry: Entry,
        mut tree_entries: HashMap<String, Entry>,
    ) {
        if parents.is_empty() {
            tree_entries.insert(entry.name.clone(), entry);
        } else {
            tree_entries.insert(parents[0].clone(), entry.clone());
            let _tree = Tree::new(tree_entries.values().cloned().collect());
            Tree::_add_entry(parents[1..].to_vec(), entry, tree_entries);
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
        let mut entries_vec: Vec<Entry> = self.entries.clone();
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
