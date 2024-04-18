use anyhow::Result;
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::workspace::{Addable, File as MyFile, FileOrDir};
use crate::{index::Checksum, lockfile::Lockfile, workspace::WorkspaceTree};

static HEADER_SIZE: usize = 12;
static ENTRY_MIN_SIZE: usize = 64;
static ENTRY_BLOCK_SIZE: usize = 8;

trait FromRaw {
    fn from_raw_parts(
        ino: u32,
        size: u32,
        mode: u16,
        uid: u32,
        gid: u32,
        ctime: u32,
        mtime: u32,
        ctime_nsec: u32,
        mtime_nsec: u32,
        dev: u32,
        flags: u16,
    ) -> Self;
}

impl FromRaw for Metadata {
    fn from_raw_parts(
        ino: u32,
        size: u32,
        mode: u16,
        uid: u32,
        gid: u32,
        ctime: u32,
        mtime: u32,
        ctime_nsec: u32,
        mtime_nsec: u32,
        dev: u32,
        flags: u16,
    ) -> Self {
        Metadata::from_raw_parts(
            ino, size, mode, uid, gid, ctime, mtime, ctime_nsec, mtime_nsec, dev, flags,
        )
    }
}

#[derive(Debug, Clone)]
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

    pub fn add(&mut self, path: &PathBuf, oid: String) {
        self.entries.add(path, oid);
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
        let num_entries = self.entries.workspace.len() as u32;
        let num_entries = num_entries.to_be_bytes().to_vec();
        writer.write(&num_entries)?;

        for (_name, entry) in &self.entries.workspace {
            match entry {
                FileOrDir::File(file) => {
                    let ctime = file.stat.ctime();
                    let ctime_nsec = file.stat.ctime_nsec();
                    let mtime = file.stat.mtime();
                    let mtime_nsec = file.stat.mtime_nsec();
                    let dev = file.stat.dev();
                    let ino = file.stat.ino();
                    let mode = file.stat.mode();
                    let uid = file.stat.uid();
                    let gid = file.stat.gid();
                    let file_size = file.stat.size();
                    let oid = hex::decode(file.oid.as_ref().expect("failed to get oid"))
                        .expect("failed to decode oid");
                    let flags = 0u16.to_be_bytes().to_vec();
                    let path = file.name.as_bytes().to_vec();

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
                    entry.extend(flags);
                    entry.extend(path);

                    writer.write(&entry)?;
                }
                _ => (),
            }
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
            dbg!("reading entry");
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

            let path = path.trim_end_matches('\0').to_owned();
            self.entries.workspace.insert(
                path.clone(),
                FileOrDir::File(MyFile {
                    name: path.clone(),
                    stat: Metadata::from_raw_parts(
                        ino,
                        file_size,
                        mode as u16,
                        uid,
                        gid,
                        ctime,
                        mtime,
                        ctime_nsec,
                        mtime_nsec,
                        dev,
                        flags,
                    ),
                    path: PathBuf::from(path),
                    oid: Some(oid),
                }),
            );
        }

        dbg!(self.entries.workspace.clone());

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
        dbg!("loading header");
        let count = self.read_header(&mut reader)?;
        dbg!("loading entries");
        self.read_entries(&mut reader, count)?;
        reader.verify_checksum()?;
        Ok(())
    }

    pub fn load_for_update(&mut self) -> Result<bool> {
        dbg!("Loading index for update");
        if self.lockfile.hold_for_update()? {
            self.load()?;
            dbg!(&self.entries.workspace);
            return Ok(true);
        }
        Ok(false)
    }
}
