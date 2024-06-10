use anyhow::Result;
use regex::Regex;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    process::exit,
};

use crate::{
    database::{Commit, Database, ParsedContent},
    lockfile::Lockfile,
    utils::write_to_stderr,
};

#[derive(Debug, Clone)]
pub struct Ref {
    revision_pattern: String,
    name: String,
}

impl Ref {
    pub fn resolve(&self, context: &Refs) -> Option<String> {
        return Some(context.get_specific_ref_content(self.name.as_str()));
    }
}
#[derive(Debug, Clone)]
pub struct Parent {
    rev: Box<Revision>,
    revision_pattern: String,
}

impl Parent {
    pub fn resolve(&self, context: &Refs) -> Option<String> {
        let ref_content = self.rev.resolve(context);
        let content = context.commit_parent(Some(&ref_content.as_ref().unwrap()));
        match content {
            Some(c) => Some(c),
            None => {
                let output = format!(
                    "fatal: Not a valid object name: '{}'",
                    self.revision_pattern
                );

                write_to_stderr(&output).unwrap();
                exit(1);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ancestor {
    rev: Box<Revision>,
    revision_pattern: String,
    num: i32,
}

impl Ancestor {
    pub fn resolve(&self, context: &Refs) -> Option<String> {
        let mut oid = self.rev.resolve(context);

        for _ in 0..self.num {
            let parent = context.commit_parent(Some(&oid.as_ref().unwrap()));
            match parent {
                Some(p) => {
                    oid = Some(p);
                }
                None => {
                    let output = format!(
                        "fatal: Not a valid object name: '{}'",
                        self.revision_pattern
                    );
                    write_to_stderr(&output).unwrap();
                    exit(1);
                }
            }
        }
        return oid;
    }
}

#[derive(Debug, Clone)]
pub enum Revision {
    Parent(Parent),
    Ancestor(Ancestor),
    Ref(Ref),
}

impl Revision {
    pub fn resolve(&self, context: &Refs) -> Option<String> {
        match self {
            Revision::Parent(parent) => parent.resolve(context),
            Revision::Ancestor(ancestor) => ancestor.resolve(context),
            Revision::Ref(r) => r.resolve(context),
        }
    }
}

pub fn parse_revision(pattern: &str, revision_pattern: &str) -> Revision {
    let parent_re = Regex::new(r"^(.+)\^$").unwrap();
    let revision_re = Regex::new(r"^(.+)~(\d+)$").unwrap();

    if parent_re.is_match(pattern) {
        let caps = parent_re.captures(pattern).unwrap();
        let rev = caps.get(1).unwrap().as_str();

        let ref_type = parse_revision(rev, revision_pattern);
        let box_ref_type = Box::new(ref_type);
        return Revision::Parent(Parent {
            rev: box_ref_type,
            revision_pattern: revision_pattern.to_string(),
        });
    } else if revision_re.is_match(pattern) {
        let caps = revision_re.captures(pattern).unwrap();
        let rev = caps.get(1).unwrap().as_str();
        let num = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();

        let ref_type = parse_revision(rev, revision_pattern);
        let box_ref_type = Box::new(ref_type);
        return Revision::Ancestor(Ancestor {
            rev: box_ref_type,
            num,
            revision_pattern: revision_pattern.to_string(),
        });
    } else {
        if pattern == "@" || pattern == "HEAD" {
            return Revision::Ref(Ref {
                name: "master".to_string(),
                revision_pattern: revision_pattern.to_string(),
            });
        }
        return Revision::Ref(Ref {
            name: pattern.to_string(),
            revision_pattern: revision_pattern.to_string(),
        });
    }
}

#[derive(Debug, Clone)]
pub struct Refs {
    pub git_path: PathBuf,
}

impl Refs {
    pub fn new(git_path: PathBuf) -> Self {
        Refs { git_path }
    }

    pub fn commit_parent(&self, oid: Option<&str>) -> Option<String> {
        match oid {
            Some(oid) => {
                let database = Database::new(self.git_path.join("objects").clone());
                let commit = database.read_object(oid);
                let commit = commit.expect("Failed to read commit");

                match commit {
                    ParsedContent::CommitContent(commit) => match commit.parent {
                        Some(parent) => Some(parent),
                        None => None,
                    },
                    _ => panic!("should not happen"),
                }
            }
            None => None,
        }
    }

    pub fn get_all_commits(&self) -> Result<Vec<Commit>> {
        let mut head = self.read_head().unwrap();
        let database = Database::new(self.git_path.join("objects").clone());
        let mut commits = vec![];

        loop {
            let commit = database.read_object(&head).unwrap();
            match commit {
                ParsedContent::CommitContent(commit) => {
                    commits.push(commit.clone());
                    match commit.parent {
                        Some(parent) => {
                            head = parent;
                        }
                        None => break,
                    }
                }
                _ => panic!("should not happen"),
            }
        }
        Ok(commits)
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
    pub fn create_branch(&self, branch_name: &str, oid: &str) -> Result<()> {
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

        self.update_ref_file(&branch_ref_path, oid)?;

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

    pub fn get_ref_content(&self) -> String {
        let ref_path = self.get_ref_path();
        if !ref_path.exists() {
            write_to_stderr("fatal: Not a valid object name: 'HEAD'").unwrap();
            exit(1);
        }
        let ref_content = fs::read_to_string(ref_path).expect("Failed to read ref");
        ref_content.trim().to_string()
    }

    pub fn get_specific_ref_content(&self, ref_name: &str) -> String {
        let ref_path = self.git_path.join("refs/heads").join(ref_name);
        if ref_path.exists() {
            let ref_content = fs::read_to_string(ref_path).expect("Failed to read ref");
            ref_content.trim().to_string()
        } else {
            let database = Database::new(self.git_path.join("objects").clone());
            let objects = database.prefix_match(ref_name);
            dbg!(objects);
            write_to_stderr(&format!("fatal: Not a valid object name: '{}'", ref_name)).unwrap();
            exit(1);
        }
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
