use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::PathBuf;

use crate::{
    index::{Checksum, Stat},
    lockfile::Lockfile,
    workspace::{Dir, File as MyFile, FileOrDir, WorkspaceTree},
};

static HEADER_SIZE: usize = 12;
static ENTRY_MIN_SIZE: usize = 64;
static ENTRY_BLOCK_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct FlatIndex {
    pub entries: BTreeMap<String, MyFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub entries: BTreeMap<String, FileOrDir>,
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

    pub fn discard_conflicts(&mut self, file: &MyFile) {
        self.entries.remove(file.path.to_str().unwrap());
    }

    pub fn build(
        entry: FileOrDir,
        parents: Vec<String>,
        path_components: Vec<String>,
        entries: &mut BTreeMap<String, FileOrDir>,
        file: &MyFile,
    ) {
        if parents.len() == 1 {
            entries.insert(parents[0].clone(), FileOrDir::File(file.clone()));
            return;
        }

        let mut components = path_components.clone();
        let component = components.remove(0);
        let mut parents = parents;
        let _ = parents.remove(0);

        if !entries.contains_key(&component) {
            entries.insert(component.clone(), entry.clone());
        }

        let parent_dir = entries.get_mut(&component).unwrap();
        match parent_dir {
            FileOrDir::Dir(dir) => {
                Index::build(entry, parents, components, &mut dir.children, file);
            }
            _ => {
                entries.insert(parents[0].clone(), entry.clone());
            }
        }
    }

    pub fn add(&mut self, file: &MyFile) {
        self.discard_conflicts(file);
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
            Index::build(
                dir_entry,
                parents,
                path_components,
                &mut self.entries,
                &file,
            );
        } else {
            self.entries.insert(
                file.path.to_str().unwrap().to_owned(),
                FileOrDir::File(file.clone()),
            );
        }

        self.changed = true;
    }

    pub fn from_flat_entries(&mut self, flat_index: &FlatIndex) {
        for (path, entry) in &flat_index.entries {
            self.add(entry);
        }
    }

    pub fn flatten_entries(entries: &BTreeMap<String, FileOrDir>, flat_index: &mut FlatIndex) {
        for (_, entry) in entries {
            match entry {
                FileOrDir::File(file) => {
                    flat_index.entries.insert(
                        file.path.as_os_str().to_str().unwrap().to_owned().clone(),
                        file.clone(),
                    );
                }
                FileOrDir::Dir(dir) => {
                    // let mut dir_entries: BTreeMap<String, FileOrDir> = BTreeMap::new();
                    Index::flatten_entries(&dir.children, flat_index);
                    // flat_index.entries.insert(
                    //     dir.path.as_os_str().to_str().unwrap().to_owned().clone(),
                    //     FileOrDir::Dir(dir.clone()),
                    // );
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

        let mut flat_index = FlatIndex {
            entries: BTreeMap::new(),
        };
        Index::flatten_entries(&self.entries, &mut flat_index);

        // pad the number of entries to 4 bytes
        let num_entries = flat_index.entries.len() as u32;
        let num_entries = num_entries.to_be_bytes().to_vec();
        writer.write(&num_entries)?;

        for (_name, entry) in &flat_index.entries {
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

    fn read_entries(&mut self, reader: &mut Checksum, count: u32) -> Result<FlatIndex> {
        println!("Reading entries");
        let mut flat_index = FlatIndex {
            entries: BTreeMap::new(),
        };
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
            let entry = MyFile {
                name: PathBuf::from(&path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                stat,
                path: PathBuf::from(path),
                oid: Some(oid.clone()),
            };
            flat_index
                .entries
                .insert(entry.path.to_str().unwrap().to_owned(), entry);
        }

        Ok(flat_index)
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
        let flat_index = self.read_entries(&mut reader, count)?;
        self.from_flat_entries(&flat_index);
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
