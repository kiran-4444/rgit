use anyhow::Result;
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

use crate::{
    index::Checksum,
    lockfile::Lockfile,
    workspace::{Dir, File as MyFile, FileOrDir, WorkspaceTree},
};

static HEADER_SIZE: usize = 12;
static ENTRY_MIN_SIZE: usize = 64;
static ENTRY_BLOCK_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Stat {
    ino: u32,
    size: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    ctime: u32,
    mtime: u32,
    ctime_nsec: u32,
    mtime_nsec: u32,
    dev: u32,
    flags: u16,
    oid: Option<String>,
    path: PathBuf,
}

static REGULAR_MODE: u32 = 0o100644;
static EXECUTABLE_MODE: u32 = 0o100755;
static MAX_PATH_SIZE: usize = 0xfff;

impl Stat {
    pub fn new(path: &PathBuf) -> Self {
        dbg!(path);
        let stat = path.metadata().expect("failed to get metadata");
        let stripped_path = path.strip_prefix(&std::env::current_dir().unwrap());
        let stripped_path = match stripped_path {
            Ok(path) => path,
            Err(_) => path,
        };

        Self {
            ino: stat.ino() as u32,
            size: stat.size() as u32,
            mode: if stat.permissions().mode() & 0o111 != 0 {
                EXECUTABLE_MODE
            } else {
                REGULAR_MODE
            } as u32,
            uid: stat.uid() as u32,
            gid: stat.gid() as u32,
            ctime: stat.ctime() as u32,
            mtime: stat.mtime() as u32,
            ctime_nsec: stat.ctime_nsec() as u32,
            mtime_nsec: stat.mtime_nsec() as u32,
            dev: stat.dev() as u32,
            flags: min(MAX_PATH_SIZE, stripped_path.to_str().unwrap().len()) as u16,
            oid: None,
            path: stripped_path.to_path_buf().clone(),
        }
    }

    pub fn from_raw(raw: &[u8]) -> Self {
        let ctime = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let ctime_nsec = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let mtime = u32::from_be_bytes([raw[8], raw[9], raw[10], raw[11]]);
        let mtime_nsec = u32::from_be_bytes([raw[12], raw[13], raw[14], raw[15]]);
        let dev = u32::from_be_bytes([raw[16], raw[17], raw[18], raw[19]]);
        let ino = u32::from_be_bytes([raw[20], raw[21], raw[22], raw[23]]);
        let mode = u32::from_be_bytes([raw[24], raw[25], raw[26], raw[27]]);
        let uid = u32::from_be_bytes([raw[28], raw[29], raw[30], raw[31]]);
        let gid = u32::from_be_bytes([raw[32], raw[33], raw[34], raw[35]]);
        let size = u32::from_be_bytes([raw[36], raw[37], raw[38], raw[39]]);
        let oid = hex::encode(&raw[40..60]);
        let flags = u16::from_be_bytes([raw[60], raw[61]]);
        let path = String::from_utf8(raw[62..].to_vec()).expect("failed to convert to string");

        Self {
            ctime,
            ctime_nsec,
            mtime,
            mtime_nsec,
            dev,
            ino,
            mode,
            uid,
            gid,
            size,
            flags,
            oid: Some(oid),
            path: PathBuf::from(path),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub entries: WorkspaceTree,
    pub lockfile: Lockfile,
    pub changed: bool,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        let lockfile = Lockfile::new(path);
        let entries = WorkspaceTree::new(None);
        Self {
            entries,
            lockfile,
            changed: false,
        }
    }

    pub fn add(&mut self, file: &MyFile, oid: String) {
        let parents =
            FileOrDir::parent_directories(&file.path).expect("failed to get parent directories");
        let path_components =
            FileOrDir::components(&file.path).expect("failed to get parent components");
        if parents.len() > 1 {
            let dir_entry = FileOrDir::Dir(Dir {
                name: parents[0].clone(),
                path: PathBuf::from(parents[0].clone()),
                children: BTreeMap::new(),
            });
            WorkspaceTree::build(
                dir_entry,
                parents,
                path_components,
                &mut self.entries.workspace,
                Some(oid),
            );
        } else {
            let file_entry = FileOrDir::File(MyFile {
                name: file.name.clone(),
                path: file.path.clone(),
                stat: file.stat.clone(),
                oid: Some(oid.clone()),
            });
            self.entries
                .workspace
                .insert(file.path.to_str().unwrap().to_owned(), file_entry);
        }

        self.changed = true;
    }

    pub fn flatten_entries(
        entries: &BTreeMap<String, FileOrDir>,
        result: &mut BTreeMap<String, MyFile>,
    ) {
        for (name, entry) in entries {
            match entry {
                FileOrDir::File(file) => {
                    result.insert(name.clone(), file.clone());
                }
                FileOrDir::Dir(dir) => {
                    Index::flatten_entries(&dir.children, result);
                }
            }
        }
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

        let mut result = BTreeMap::new();
        Index::flatten_entries(&self.entries.workspace, &mut result);

        // pad the number of entries to 4 bytes
        let num_entries = result.len() as u32;
        let num_entries = num_entries.to_be_bytes().to_vec();
        writer.write(&num_entries)?;

        for (_name, entry) in &result {
            let ctime = entry.stat.ctime;
            let ctime_nsec = entry.stat.ctime_nsec;
            let mtime = entry.stat.mtime;
            let mtime_nsec = entry.stat.mtime_nsec;
            let dev = entry.stat.dev;
            let ino = entry.stat.ino;
            let mode = entry.stat.mode;
            let uid = entry.stat.uid;
            let gid = entry.stat.gid;
            let file_size = entry.stat.size;
            let oid = hex::decode(entry.oid.as_ref().expect("failed to get oid"))
                .expect("failed to decode oid");
            let flags = entry.stat.flags;
            let path = entry.path.as_os_str().to_str().unwrap().as_bytes().to_vec();

            let mut entry = vec![];
            entry.extend(ctime.to_be_bytes().to_vec());
            entry.extend(ctime_nsec.to_be_bytes().to_vec());
            entry.extend(mtime.to_be_bytes().to_vec());
            entry.extend(mtime_nsec.to_be_bytes().to_vec());
            entry.extend(dev.to_be_bytes().to_vec());
            entry.extend(ino.to_be_bytes().to_vec());
            entry.extend(mode.to_be_bytes().to_vec());
            entry.extend(uid.to_be_bytes().to_vec());
            entry.extend(gid.to_be_bytes().to_vec());
            entry.extend(file_size.to_be_bytes().to_vec());
            entry.extend(oid);
            entry.extend(flags.to_be_bytes().to_vec());
            entry.extend(path);

            if entry.len() % 8 == 0 && entry[entry.len() - 1] == 0 {
                writer.write(&entry)?;
            } else {
                let padding_length = 8 - (entry.len() % 8);
                let padding = vec![0; padding_length];
                entry.extend(&padding);
                writer.write(&entry)?;
            }
            println!("{:?}", entry);
        }

        writer.write_checksum()?;
        self.lockfile.commit()?;
        self.changed = false;
        Ok(())
    }

    fn clear(&mut self) {
        self.entries.workspace.clear();
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

            let stat = Stat::from_raw(&entry);
            let oid = hex::encode(&entry[40..60]);
            let path =
                String::from_utf8(entry[62..].to_vec()).expect("failed to convert to string");

            let path = path.trim_end_matches('\0').to_owned();
            self.add(
                &MyFile {
                    name: path.clone(),
                    stat,
                    path: PathBuf::from(path),
                    oid: Some(oid.clone()),
                },
                oid.clone(),
            );
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
