use anyhow::Result;
use clap::{arg, Parser};
use std::{env, path::Path};

use crate::{
    command::init::{check_if_git_dir_exists, construct_git_path},
    database,
    objects::*,
    refs,
    utils::{write_to_stderr, write_to_stdout},
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
        let mut index = Index::new(git_path.join("index"));

        index.load()?;
        let entries = index
            .entries
            .values()
            .cloned()
            .collect::<Vec<index::Entry>>();
        println!("{:?}", entries);
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
