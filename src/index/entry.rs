use anyhow::Result;
use std::cmp::min;
use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum IndexEntry {
    Entry(FileEntry),
    TreeEntry,
}

static REGULAR_MODE: u32 = 0o100644;
static EXECUTABLE_MODE: u32 = 0o100755;
static MAX_PATH_SIZE: usize = 0xfff;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileEntry {
    pub oid: String,
    pub ctime: u32,
    pub ctime_nsec: u32,
    pub mtime: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub flags: u16,
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(name: &PathBuf, oid: String, stat: Metadata) -> Self {
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
        let flags = min(
            MAX_PATH_SIZE,
            name.as_os_str()
                .to_str()
                .expect("failed to convert path to str")
                .len(),
        ) as u16;
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

    pub fn convert(&self) -> Vec<u8> {
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
        data.extend(
            self.path
                .as_os_str()
                .to_str()
                .expect("failed to covert path to str")
                .as_bytes(),
        );
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
