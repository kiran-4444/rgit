use std::iter::zip;

use itertools::Itertools;

use super::{storable::Storable, Entry};

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
}

impl Storable for Tree {
    fn blob_type(&self) -> &str {
        "tree"
    }

    fn set_oid(&mut self, oid: &str) {
        self.oid = Some(oid.to_string());
    }

    fn data(&self) -> String {
        let mut entries_vec: Vec<Entry> = self.entries.clone();
        entries_vec.sort_by(|a, b| a.name.cmp(&b.name));

        let mut hex_oids: Vec<Vec<u8>> = Vec::new();
        let mut entries = entries_vec
            .iter()
            .map(|entry| {
                let mut output: Vec<&[u8]> = Vec::new();

                output.push(self.mode.as_bytes());
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
