use anyhow::Result;
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

use crate::lockfile::Lockfile;

use super::Checksum;

static REGULAR_MODE: u32 = 0o100644;
static EXECUTABLE_MODE: u32 = 0o100755;
static MAX_PATH_SIZE: usize = 0xfff;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub oid: String,
    ctime: u32,
    ctime_nsec: u32,
    mtime: u32,
    mtime_nsec: u32,
    dev: u32,
    ino: u32,
    pub mode: u32,
    uid: u32,
    gid: u32,
    file_size: u32,
    flags: u16,
    pub path: String,
}

impl Entry {
    fn new(name: String, oid: String, stat: Metadata) -> Self {
        let ctime = stat.ctime() as u32;
        let ctime_nsec = stat.ctime_nsec() as u32;
        let mtime = stat.mtime() as u32;
        let mtime_nsec = stat.mtime_nsec() as u32;
        let dev = stat.dev() as u32;
        let ino = stat.ino() as u32;
        let is_executable = stat.permissions().mode() & 0o111 != 0;
        let mode = if is_executable {
            EXECUTABLE_MODE
        } else {
            REGULAR_MODE
        } as u32;
        let uid = stat.uid() as u32;
        let gid = stat.gid() as u32;
        let file_size = stat.len() as u32;
        let flags = min(MAX_PATH_SIZE, name.len()) as u16;
        let path = name.clone();
        Self {
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
            path,
        }
    }

    fn convert(&self) -> Vec<u8> {
        let mut data = vec![];
        data.extend(&self.ctime.to_be_bytes());
        data.extend(&self.ctime_nsec.to_be_bytes());
        data.extend(&self.mtime.to_be_bytes());
        data.extend(&self.mtime_nsec.to_be_bytes());
        data.extend(&self.dev.to_be_bytes());
        data.extend(&self.ino.to_be_bytes());
        data.extend(&self.mode.to_be_bytes());
        data.extend(&self.uid.to_be_bytes());
        data.extend(&self.gid.to_be_bytes());
        data.extend(&self.file_size.to_be_bytes());
        let decoded_hex = hex::decode(self.oid.clone()).expect("failed to decode hex");
        data.extend(decoded_hex);
        data.extend(self.flags.to_be_bytes());
        data.extend(self.path.as_bytes());
        if data.len() % 8 == 0 && data[data.len() - 1] == 0 {
            return data;
        }

        let padding_length = 8 - (data.len() % 8);
        let padding = vec![0; padding_length];
        data.extend(&padding);
        data
    }

    pub fn parent_directories(&self) -> Result<Vec<String>> {
        let mut parents = Vec::new();
        let components = PathBuf::from(&self.path)
            .components()
            .map(|c| {
                Ok(c.as_os_str()
                    .to_str()
                    .expect("failed to convert path to str")
                    .to_owned())
            })
            .collect::<Result<Vec<_>>>()?;
        let mut current_path = String::new();
        for part in components.iter().take(components.len() - 1) {
            current_path.push_str(part);
            parents.push(current_path.clone());
            current_path.push('/');
        }
        Ok(parents)
    }
}

static HEADER_SIZE: usize = 12;
static ENTRY_MIN_SIZE: usize = 64;
static ENTRY_BLOCK_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub struct Index {
    pub entries: BTreeMap<String, Entry>,
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

    fn discard_conflicts(&mut self, entry: &Entry) {
        let mut parents = entry
            .parent_directories()
            .expect("failed to get parent directories");
        parents.reverse();
        println!("parents: {:?}", parents);
        println!("entries: {:?}", self.entries);

        for parent in parents {
            let parent = parent.trim_end_matches('\0').to_owned();
            println!("Checking parent: {:?}", parent);

            if self.entries.contains_key(&parent) {
                self.entries.remove(&parent);
            }
        }
    }

    pub fn add(&mut self, path: &PathBuf, oid: String, stat: Metadata) {
        let name = path
            .to_str()
            .expect("failed to convert path to str")
            .to_owned();
        // let name = name.trim_start_matches("\0").to_owned();
        let entry = Entry::new(name.to_owned(), oid, stat);
        self.discard_conflicts(&entry);
        self.entries.insert(name, entry);
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
            let content = entry.convert();
            writer.write(&content)?;
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
            let entry = Entry {
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
            self.entries.insert(path, entry);
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
