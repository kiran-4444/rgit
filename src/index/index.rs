use anyhow::Result;
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::path::PathBuf;

use crate::{index::Checksum, index::FileEntry, lockfile::Lockfile};

use super::entry::IndexEntry;

static HEADER_SIZE: usize = 12;
static ENTRY_MIN_SIZE: usize = 64;
static ENTRY_BLOCK_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub struct Index {
    pub entries: BTreeMap<String, IndexEntry>,
    pub lockfile: Lockfile,
    pub changed: bool,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        let lockfile = Lockfile::new(path);
        let entries = BTreeMap::new();
        Self {
            entries,
            lockfile,
            changed: false,
        }
    }

    /// Discard any entries that conflict with the given entry.
    /// This is used to handle cases where a file is being added to the index and a directory
    /// with the same name exists in the index or vice versa.
    /// # Example:
    /// ```bash
    /// git add foo.txt/bar
    /// rm -rf foo.txt
    /// git add foo.txt
    /// ```
    /// In this case, the directory foo.txt is removed and a file with the same name is added.
    /// The directory foo.txt should be removed from the index.
    /// ```bash
    /// git add foo.txt
    /// rm foo.txt
    /// mkdir foo.txt
    /// touch foo.txt/bar
    /// git add foo.txt
    /// ```
    /// In this case, the file foo.txt is removed and a directory with the same name is created.
    /// The file foo.txt should be removed from the index.
    fn discard_conflicts(&mut self, entry: &FileEntry) {
        let mut parents = entry
            .parent_directories()
            .expect("failed to get parent directories");

        // if the entry is a file, we need to add this file to the parents
        // handles the case where a directory with the same name as the file exists in the index
        // and the file is being added to the index as well
        if parents.is_empty() {
            parents.push(entry.path.clone());
        }

        for parent in parents {
            let parent = parent.trim_end_matches('\0').to_owned();
            for (name, entry) in self.entries.clone() {
                match entry {
                    // if the entry is a file and the path starts with the parent directory
                    // remove the entry from the index
                    IndexEntry::Entry(entry) => {
                        if entry.path.starts_with(&parent) {
                            self.entries.remove(&name);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add(&mut self, path: &PathBuf, oid: String, stat: Metadata) {
        let name = path
            .to_str()
            .expect("failed to convert path to str")
            .to_owned();
        let entry = FileEntry::new(name.to_owned(), oid, stat);
        // self.discard_conflicts(&entry);
        self.entries.insert(name, IndexEntry::Entry(entry));
        self.changed = true;
    }

    pub fn write_updates(&mut self) -> Result<()> {
        if !self.changed {
            self.lockfile.rollback()?;
        }

        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(
            self.lockfile
                .lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref"),
        )?;
        let mut writer = Checksum::new(&mut file);

        let signature = "DIRC".to_owned().bytes().collect::<Vec<u8>>();
        writer.write(&signature)?;

        // pad the version to 4 bytes
        let version = 2u32.to_be_bytes().to_vec();
        writer.write(&version)?;

        // pad the number of entries to 4 bytes
        let num_entries = self.entries.len() as u32;
        let num_entries = num_entries.to_be_bytes().to_vec();
        writer.write(&num_entries)?;

        for (_name, entry) in &self.entries {
            match entry {
                IndexEntry::Entry(entry) => {
                    let content = entry.convert();
                    writer.write(&content)?;
                }
                _ => {
                    panic!("Invalid entry type");
                }
            }
        }

        writer.write_checksum()?;
        self.lockfile.commit()?;
        self.changed = false;
        Ok(())
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.changed = false;
    }

    fn open_index_file(&self) -> Result<Option<File>> {
        if self.lockfile.file_path.exists() {
            let file = File::open(&self.lockfile.file_path)?;
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    fn read_header(&mut self, reader: &mut Checksum) -> Result<u32> {
        let header = reader.read(HEADER_SIZE)?;
        let signature = &header[0..4];
        if signature != b"DIRC" {
            anyhow::bail!("Invalid index file signature");
        }
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
        if version != 2 {
            anyhow::bail!("Unsupported index file version");
        }
        let num_entries = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
        Ok(num_entries)
    }

    fn read_entries(&mut self, reader: &mut Checksum, count: u32) -> Result<()> {
        for _ in 0..count {
            let mut entry = reader.read(ENTRY_MIN_SIZE)?;
            loop {
                if entry[entry.len() - 1] == 0 as u8 {
                    break;
                }
                let padding = reader.read(ENTRY_BLOCK_SIZE)?;
                entry.extend(padding);
            }

            let ctime = u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let ctime_nsec = u32::from_be_bytes([entry[4], entry[5], entry[6], entry[7]]);
            let mtime = u32::from_be_bytes([entry[8], entry[9], entry[10], entry[11]]);
            let mtime_nsec = u32::from_be_bytes([entry[12], entry[13], entry[14], entry[15]]);
            let dev = u32::from_be_bytes([entry[16], entry[17], entry[18], entry[19]]);
            let ino = u32::from_be_bytes([entry[20], entry[21], entry[22], entry[23]]);
            let mode = u32::from_be_bytes([entry[24], entry[25], entry[26], entry[27]]);
            let uid = u32::from_be_bytes([entry[28], entry[29], entry[30], entry[31]]);
            let gid = u32::from_be_bytes([entry[32], entry[33], entry[34], entry[35]]);
            let file_size = u32::from_be_bytes([entry[36], entry[37], entry[38], entry[39]]);
            let oid = hex::encode(&entry[40..60]);
            let flags = u16::from_be_bytes([entry[60], entry[61]]);
            let path = String::from_utf8(entry[62..].to_vec())?;
            let entry = FileEntry {
                oid,
                ctime,
                ctime_nsec,
                mtime,
                mtime_nsec,
                dev,
                ino,
                mode,
                uid,
                gid,
                file_size,
                flags,
                path: path.clone(),
            };
            let path = path.trim_end_matches('\0').to_owned();
            self.entries.insert(path, IndexEntry::Entry(entry));
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        self.clear();
        let file = self.open_index_file()?;
        if file.is_none() {
            return Ok(());
        }
        let mut file = file.expect("failed to get file content");
        let mut reader = Checksum::new(&mut file);
        let count = self.read_header(&mut reader)?;
        self.read_entries(&mut reader, count)?;
        reader.verify_checksum()?;
        Ok(())
    }

    pub fn load_for_update(&mut self) -> Result<bool> {
        if self.lockfile.hold_for_update()? {
            self.load()?;
            return Ok(true);
        }
        Ok(false)
    }
}
