use sha1::{Digest, Sha1};

use crate::lockfile::Lockfile;
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

static REGULAR_MODE: u32 = 0o100644;
static EXECUTABLE_MODE: u32 = 0o100755;
static MAX_PATH_SIZE: usize = 0xfff;

#[derive(Debug)]
struct Entry {
    oid: String,
    ctime: i64,
    ctime_nsec: i64,
    mtime: i64,
    mtime_nsec: i64,
    dev: u64,
    ino: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    file_size: u64,
    flags: u16,
    path: String,
}

impl Entry {
    fn new(name: String, oid: String, stat: Metadata) -> Self {
        let ctime = stat.ctime();
        let ctime_nsec = stat.ctime_nsec();
        let mtime = stat.mtime();
        let mtime_nsec = stat.mtime_nsec();
        let dev = stat.dev();
        let ino = stat.ino();
        let is_executable = stat.permissions().mode() & 0o111 != 0;
        let mode = if is_executable {
            EXECUTABLE_MODE
        } else {
            REGULAR_MODE
        };
        let uid = stat.uid();
        let gid = stat.gid();
        let file_size = stat.len();
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
        data.extend(&(self.ctime as u32).to_be_bytes());
        data.extend(&(self.ctime_nsec as u32).to_be_bytes());
        data.extend(&(self.mtime as u32).to_be_bytes());
        data.extend(&(self.mtime_nsec as u32).to_be_bytes());
        data.extend(&(self.dev as u32).to_be_bytes());
        data.extend(&(self.ino as u32).to_be_bytes());
        data.extend(&(self.mode as u32).to_be_bytes());
        data.extend(&(self.uid as u32).to_be_bytes());
        data.extend(&(self.gid as u32).to_be_bytes());
        data.extend(&(self.file_size as u32).to_be_bytes());
        let decoded_hex = hex::decode(self.oid.clone()).expect("failed to decode hex");
        data.extend(decoded_hex);
        data.extend(self.flags.to_be_bytes());
        data.extend(self.path.as_bytes());
        data
    }
}

#[derive(Debug)]
pub struct Index {
    entries: BTreeMap<String, Entry>,
    lockfile: Lockfile,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        let lockfile = Lockfile::new(path);
        let entries = BTreeMap::new();
        Self { entries, lockfile }
    }

    pub fn add(&mut self, path: &PathBuf, oid: String, stat: Metadata) {
        let name = path
            .to_str()
            .expect("failed to convert path to str")
            .to_owned();
        let entry = Entry::new(name.to_owned(), oid, stat);
        self.entries.insert(name, entry);
    }

    pub fn write_updates(&mut self) {
        if !self.lockfile.hold_for_update() {
            panic!("failed to hold lockfile for update");
        }
        let mut hasher = Sha1::new();

        let signature = "DIRC".to_owned().bytes().collect::<Vec<u8>>();

        // pad the version to 4 bytes
        let version = 2u32.to_be_bytes().to_vec();

        // pad the number of entries to 4 bytes
        let num_entries = self.entries.len() as u32;
        let num_entries = num_entries.to_be_bytes().to_vec();

        self.lockfile.write(&signature);
        self.lockfile.write(&version);
        self.lockfile.write(&num_entries);
        hasher.update(&signature);
        hasher.update(&version);
        hasher.update(&num_entries);
        self.entries.iter().for_each(|(_name, entry)| {
            let mut content = entry.convert();
            self.lockfile.write(&content);
            // concatenate null bytes until the next 8-byte boundary
            let padding_length = 8 - (content.len() % 8);
            let padding = vec![0; padding_length];
            content.extend(&padding);
            self.lockfile.write(&padding);

            hasher.update(&content);
        });

        self.lockfile.write(&hasher.finalize().to_vec());
        self.lockfile.commit();
    }
}
