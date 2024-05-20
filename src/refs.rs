use anyhow::Result;
use regex::Regex;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    process::exit,
};

use crate::{lockfile::Lockfile, utils::write_to_stderr};

#[derive(Debug, Clone)]
pub struct Refs {
    pub git_path: PathBuf,
}

impl Refs {
    pub fn new(git_path: PathBuf) -> Self {
        Refs { git_path }
    }

    pub fn update_head(&self, oid: &str) -> Result<()> {
        self.update_ref_file(&self.get_ref_path(), oid)
    }

    pub fn ref_path(&self) -> PathBuf {
        let ref_path = self.get_ref_path();
        ref_path
    }

    pub fn get_branch_name(&self) -> String {
        let ref_path = self.get_ref_path();
        let branch_name = ref_path
            .file_name()
            .expect("Failed to get branch name")
            .to_str()
            .expect("Failed to convert branch name to string")
            .to_string();
        branch_name
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branch_ref_path = self.git_path.join("refs/heads");
        let branch_refs = fs::read_dir(branch_ref_path)?;
        let mut branches = vec![];
        for branch_ref in branch_refs {
            let branch_ref = branch_ref?;
            let branch_name = branch_ref
                .file_name()
                .into_string()
                .expect("Failed to convert branch name to string");

            branches.push(branch_name);
        }
        branches.sort();
        Ok(branches)
    }

    /// Create a new branch
    /// # Arguments
    /// * `branch_name` - The name of the branch to create
    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let branch_ref_path = self.git_path.join("refs/heads").join(branch_name);

        if branch_ref_path.exists() {
            write_to_stderr("fatal: A branch with that name already exists")?;
            exit(1);
        }

        // check if the branch name is valid
        // https://git-scm.com/docs/git-check-ref-format
        let invalid_name_pattern = r"(?x)
        ^\.
        | \/ \.
        | \. \.
        | ^\/
        | \/$
        | \.lock$
        | @\{
        | [\x00-\x20*:?\[\\^~\x7f]
        ";

        if Regex::new(invalid_name_pattern)
            .unwrap()
            .is_match(branch_name)
        {
            write_to_stderr(&format!("fatal: invalid branch name: {}", branch_name))?;
            exit(1);
        }

        // check if there exists a HEAD ref
        let head_ref_path = self.get_ref_path();
        if !head_ref_path.exists() {
            write_to_stderr("fatal: Not a valid object name: 'HEAD'")?;
            exit(1);
        }

        let head_ref_content = self.read_ref_content();
        self.update_ref_file(&branch_ref_path, &head_ref_content)?;

        Ok(())
    }

    fn update_ref_file(&self, ref_path: &PathBuf, oid: &str) -> Result<()> {
        let mut lockfile = Lockfile::new(ref_path.clone());
        match lockfile.hold_for_update()? {
            false => {
                write_to_stderr("fatal: Unable to create lock on HEAD")?;
                exit(1);
            }
            true => (),
        }

        lockfile.write(oid.as_bytes())?;
        lockfile.write(b"\n")?;
        lockfile.commit()?;
        Ok(())
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
