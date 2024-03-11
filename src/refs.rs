use std::path::PathBuf;

use crate::lockfile::Lockfile;

#[derive(Debug, Clone)]
pub struct Refs {
    pub git_path: std::path::PathBuf,
}

impl Refs {
    pub fn new(git_path: PathBuf) -> Self {
        Refs { git_path }
    }

    pub fn update_head(&self, oid: &str) {
        let mut lockfile = Lockfile::new(self.git_path.join("HEAD"));

        match lockfile.hold_for_update() {
            false => {
                eprintln!("fatal: Unable to create lock on HEAD");
                std::process::exit(1);
            }
            true => (),
        }

        lockfile.write(oid.as_bytes());
        lockfile.write(b"\n");
        lockfile.commit();
    }

    pub fn head_path(&self) -> std::path::PathBuf {
        self.git_path.join("HEAD")
    }

    pub fn read_head(&self) -> Option<String> {
        match self.head_path().exists() {
            true => Some(std::fs::read_to_string(self.head_path()).unwrap()),
            false => None,
        }
    }
}
