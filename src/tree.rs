use std::iter::zip;

use crate::entry::Entry;
use itertools::Itertools;

#[derive(Debug)]
pub struct Tree {
    pub mode: String,
    pub oid: Option<String>,
    pub entries: Vec<Entry>,
}

impl Tree {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self {
            mode: "100644".to_owned(),
            oid: None,
            entries,
        }
    }

    pub fn blob_type(&self) -> &str {
        "tree"
    }

    pub fn set_oid(&mut self, oid: &str) {
        self.oid = Some(oid.to_string());
    }

    pub fn tree_content(&self) -> Vec<u8> {
        let mut entries_vec: Vec<Entry> = self.entries.clone();
        entries_vec.sort_by(|a, b| a.name.cmp(&b.name));

        let mut hex_oids: Vec<String> = Vec::new();
        let mut entries = entries_vec
            .iter()
            .map(|entry| {
                let mut output: Vec<&[u8]> = Vec::new();

                output.push(self.mode.as_bytes());

                let entry_name_bytes = entry.name.as_bytes();
                output.push(entry_name_bytes);

                let null_byte_array = &[b'\x00'];
                output.push(null_byte_array);

                // println!("output: {:?}", output);
                // let oid_bytes: &[u8] = &hex::decode(&entry.oid).unwrap();
                // output.push(oid_bytes);

                // oid_bytes = hex::decode(&entry.oid).unwrap();
                // output.push(hex::encode(&entry.oid).as_bytes());
                hex_oids.push(hex::encode(&entry.oid));
                output
            })
            .collect_vec();

        // println!("entries: {:?}", entries);

        // concatenate the entries
        let mut concatenated_entries: Vec<u8> = Vec::new();
        for (entry, hex_oid) in zip(&mut entries, &hex_oids) {
            entry.push(&hex_oid.as_bytes());
            for e in entry.clone() {
                concatenated_entries.extend(e);
            }
        }

        concatenated_entries
    }
}
