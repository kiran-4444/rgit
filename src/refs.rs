use anyhow::Result;
use std::path::PathBuf;

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
        let mut lockfile = Lockfile::new(self.git_path.join("HEAD"));

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

    pub fn head_path(&self) -> std::path::PathBuf {
        self.git_path.join("HEAD")
    }

    pub fn read_head(&self) -> Option<String> {
        match self.head_path().exists() {
            true => {
                let head_content =
                    std::fs::read_to_string(self.head_path()).expect("Failed to read HEAD");
                if head_content.starts_with("ref: ") {
                    return None;
                }
                Some(head_content)
            }
            false => None,
        }
    }
}
