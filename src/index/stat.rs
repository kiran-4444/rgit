use std::cmp::min;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

use crate::database::FileMode;

#[derive(Debug, Clone, PartialEq)]
pub struct Stat {
    pub ino: u32,
    pub size: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub ctime: u32,
    pub mtime: u32,
    pub ctime_nsec: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub flags: u16,
    pub oid: Option<String>,
    pub path: PathBuf,
}

static MAX_PATH_SIZE: usize = 0xfff;

impl Default for Stat {
    fn default() -> Self {
        Self {
            ino: 0,
            size: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            ctime: 0,
            mtime: 0,
            ctime_nsec: 0,
            mtime_nsec: 0,
            dev: 0,
            flags: 0,
            oid: None,
            path: PathBuf::new(),
        }
    }
}

impl Stat {
    pub fn new(path: &PathBuf) -> Self {
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
                FileMode::Executable
            } else {
                FileMode::Regular
            }
            .into(),
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
        let path = String::from_utf8(raw[62..].to_vec())
            .expect("failed to convert to string")
            .trim_matches('\0')
            .to_owned();

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
