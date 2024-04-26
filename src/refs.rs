use anyhow::Result;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use crate::{lockfile::Lockfile, utils::write_to_stderr};

#[derive(Debug, Clone)]
pub struct Refs {
    pub git_path: std::path::PathBuf,
}

impl Refs {
    pub fn new(git_path: PathBuf) -> Self {
        Refs { git_path }
    }

    pub fn update_head(&self, oid: &str) -> Result<()> {
        let branch_ref_path = self.get_ref_path();
        let mut lockfile = Lockfile::new(branch_ref_path);
        match lockfile.hold_for_update()? {
            false => {
                write_to_stderr("fatal: Unable to create lock on HEAD")?;
                std::process::exit(1);
            }
            true => (),
        }

        lockfile.write(oid.as_bytes())?;
        lockfile.write(b"\n")?;
        lockfile.commit()?;
        Ok(())
    }

    pub fn ref_path(&self) -> PathBuf {
        let ref_path = self.get_ref_path();
        ref_path
    }

    fn get_branch_name(&self) -> String {
        let ref_path = self.get_ref_path();
        let branch_name = ref_path
            .file_name()
            .expect("Failed to get branch name")
            .to_str()
            .expect("Failed to convert branch name to string")
            .to_string();
        branch_name
    }

    fn read_ref_content(&self) -> String {
        let ref_path = self.get_ref_path();
        if !ref_path.exists() {
            // create branch ref file
            let branch_name = self.get_branch_name();
            let branch_ref_path = self.git_path.join("refs/heads").join(branch_name);
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(&branch_ref_path)
                .expect("Failed to create branch ref file");
            return "".to_string();
        }
        let ref_content = std::fs::read_to_string(ref_path).expect("Failed to read ref");
        ref_content.trim().to_string()
    }

    /// Get the path of the ref pointed to by HEAD
    fn get_ref_path(&self) -> PathBuf {
        let head_content =
            fs::read_to_string(self.git_path.join("HEAD")).expect("Failed to read HEAD");
        if head_content.starts_with("ref: ") {
            let ref_path = head_content.trim().split(" ").collect::<Vec<&str>>()[1];
            return self.git_path.join(ref_path);
        } else {
            panic!("HEAD is not a ref");
        }
    }

    pub fn read_head(&self) -> Option<String> {
        match self.ref_path().exists() {
            true => Some(self.read_ref_content()),
            false => None,
        }
    }
}
