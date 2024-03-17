use anyhow::Result;
use clap::{arg, Parser};
use std::path::PathBuf;
use std::{env, path::Path};

use crate::{
    command::init::{check_if_git_dir_exists, construct_git_path},
    database,
    objects::*,
    refs,
    utils::{write_to_stderr, write_to_stdout},
    workspace,
};

#[derive(Parser, Debug, PartialEq)]
pub struct CommitCMD {
    #[arg(short)]
    message: String,
}

impl CommitCMD {
    pub fn run(&self) -> Result<()> {
        match check_if_git_dir_exists(&Path::new("."))? {
            false => {
                write_to_stderr(
                    "fatal: not a git repository (or any of the parent directories): .rgit",
                )?;
                std::process::exit(1);
            }
            true => (),
        }
        let git_path = construct_git_path(&Path::new("."))?;
        let refs = refs::Refs::new(git_path.clone());
        let object_store = git_path.join("objects");
        let mut db = database::Database::new(object_store);
        let workspace = workspace::Workspace::new(PathBuf::from("."));
        let workspace_entries = workspace.list_files(std::env::current_dir()?)?;
        let entries = workspace_entries
            .iter()
            .map(|entry| {
                // check if the file is a directory
                let entry_name = entry.name.to_owned();
                let entry_mode = entry.mode.to_owned();
                let data = std::fs::read_to_string(&entry_name).expect("failed to read file");
                let mut blob = Blob::new(data.to_owned());
                db.store(&mut blob)?;
                Ok(Entry::new(
                    entry_name
                        .to_str()
                        .expect("failed to convert path to str")
                        .to_owned(),
                    blob.oid.expect("failed to get oid").to_owned(),
                    entry_mode.to_owned(),
                ))
            })
            .collect::<Result<Vec<Entry>>>()?;

        let mut root = Tree::build(entries.clone())?;
        root.traverse(&mut db)?;
        db.store(&mut root)?;

        let (name, email) = self.get_config()?;
        let author = Author::new(&name, &email);

        let parent = refs.read_head();
        let message = self.message.clone();
        let mut commit = Commit::new(
            parent.to_owned(),
            root.oid.expect("OID not found"),
            author,
            &message,
        );
        db.store(&mut commit)?;

        let commit_oid = commit.oid.expect("Failed to get commit oid").clone();
        refs.update_head(&commit_oid)?;

        match parent {
            Some(_) => {
                write_to_stdout(&format!("{} {}", commit_oid, commit.message))?;
                Ok(())
            }
            None => {
                write_to_stdout(&format!("(root-commit) {} {}", commit_oid, commit.message))?;
                Ok(())
            }
        }
    }
    /// Get the author name and email from the environment variables
    fn get_config(&self) -> Result<(String, String)> {
        Ok((
            env::var("RGIT_AUTHOR_NAME")?,
            env::var("RGIT_AUTHOR_EMAIL")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_retrieves_author_details_from_env_vars() -> Result<()> {
        std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
        std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");

        let commit_cmd = CommitCMD {
            message: "".to_string(),
        };
        let (name, email) = commit_cmd.get_config()?;

        assert_eq!(name, "Test Author");
        assert_eq!(email, "test@example.com");

        Ok(())
    }
}
