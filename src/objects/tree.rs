use std::{collections::HashMap, iter::zip};

use itertools::Itertools;

use super::{storable::Storable, Entry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryOrTree {
    Entry(Entry),
    Tree(Tree),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub mode: Option<String>,
    pub oid: Option<String>,
    pub entries: HashMap<String, EntryOrTree>,
}

impl Tree {
    pub fn new(entries: HashMap<String, EntryOrTree>) -> Self {
        Self {
            mode: None,
            oid: None,
            entries,
        }
    }

    pub fn build(entries: Vec<Entry>) -> Tree {
        let root = Tree::new(HashMap::new());
        let mut hash_entries = root.entries.clone();
        for entry in entries {
            println!(
                "Starting to add entry
            {:?}",
                entry.clone()
            );
            root.add_entry(entry.parent_directories(), entry, &mut hash_entries);
        }

        dbg!(hash_entries.clone());
        root
    }

    pub fn add_entry(
        &self,
        parents: Vec<String>,
        entry: Entry,
        tree_entries: &mut HashMap<String, EntryOrTree>,
    ) {
        // println!();
        // println!("=====================");
        // dbg!(parents.clone());
        // dbg!(entry.clone());
        // dbg!(tree_entries.clone());
        if parents.is_empty() {
            tree_entries.insert(entry.name.clone(), EntryOrTree::Entry(entry));
            // println!("After insert: {:?}", tree_entries.clone());
        } else {
            let tree_entry_copy = tree_entries.clone();
            let result = tree_entry_copy.get(&entry.name.clone());
            dbg!(result.clone());
            match result {
                Some(EntryOrTree::Tree(tree)) => {
                    println!("Inside tree");
                    tree_entries.insert(entry.name.clone(), EntryOrTree::Tree(tree.clone()));
                    tree.add_entry(parents[1..].to_vec(), entry.clone(), tree_entries);
                }
                _ => {
                    // let mut tree_entries = HashMap::new();
                    // println!("Inserting key: {}", parents[0]);
                    // println!(
                    //     "Inserting value: {:?}",
                    //     EntryOrTree::Tree(Tree::new(tree_entries.clone()))
                    // );

                    tree_entries.insert(
                        parents[0].clone(),
                        EntryOrTree::Tree(Tree::new(HashMap::new())),
                    );

                    // println!("After insert: {:?}", tree_entries.clone());
                    println!("=====================");
                    let tree = Tree::new(HashMap::new());
                    tree.add_entry(parents[1..].to_vec(), entry.clone(), tree_entries);
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
